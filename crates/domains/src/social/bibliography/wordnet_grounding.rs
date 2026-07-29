//! The FRBR/B2 keystone: grounding `Genre`'s abstract Group-3-adjacent
//! classification concept in WordNet's concrete, LOADED `domain_topic` /
//! `has_domain_topic` relations -- the bridge that lets a real WN-LMF
//! domain-topic edge (e.g. "law" domains "letters patent") be read as a
//! witness of the FRBR theory in `ontology.rs`, not just a same-named
//! coincidence. Mirrors the shape of the sibling A4 keystone,
//! `formal::mereology::wordnet_grounding`: a standalone Option-returning
//! classifier, a keystone axiom proving the classifier's claims, and a
//! separate axiom proving the composition holds -- once against a small
//! hand fixture (representative, not exhaustive) and once against the
//! REAL loaded English WordNet corpus (a generated sweep, not a fixed
//! count).
//!
//! # A corrected direction, and a dropped invented correspondence
//!
//! Two real findings from re-verifying this module against the loaded
//! corpus (2026-07-21), both incorporated below rather than patched over
//! (`feedback_fix_via_research`):
//!
//! 1. **`has_domain_topic`/`domain_topic` direction.** The accessor docs on
//!    [`English`](crate::cognitive::linguistics::english::English) previously
//!    read "term → domain" for `has_domain_topic`, matching this module's
//!    ORIGINAL hand fixture (`epic --has_domain_topic--> literature`). Direct
//!    inspection of the loaded `english-wordnet-2025.xml` shows the opposite:
//!    the DOMAIN synset (e.g. `oewn-08458195-n` "law", `oewn-06376048-n`
//!    "literature") carries the `has_domain_topic` edges to its many members
//!    (`oewn-06563618-n` "letters patent" among law's ~30; 19 for literature),
//!    while the MEMBER carries the inverse `domain_topic` edge back to its
//!    domain. `English`'s doc comments and this module's fixture are now
//!    corrected to match (see `english/ontology.rs` `WordnetRelations`
//!    doc, 2026-07-21).
//! 2. **No loaded `exemplifies` edge exists anywhere near "genre" or
//!    "literature".** The ORIGINAL `WorkGenreGroundsInLoadedWordNetRelations`
//!    axiom asserted `epic --exemplifies--> narrative_poem` as a
//!    representative fixture for Genre's claimed exemplifies grounding —
//!    but that pairing does not exist in the loaded OEWN 2025 corpus (real
//!    epic-poem synset `oewn-06391344-n` carries only `hypernym`/`hyponym`
//!    edges; genre is modelled taxonomically, not via instance-of). A full
//!    sweep of the loaded corpus (184 genre-subtree descendants, the
//!    complete `has_domain_topic("literature")` membership, and every
//!    `exemplifies` edge in the corpus, 1639 of them) found ZERO real
//!    `exemplifies`/`is_exemplified_by` edge touching the genre/literature
//!    area. Per `feedback_literature_or_remove` ("never invent concept
//!    names") and `feedback_ontological_assertions`, Genre's WordNet
//!    grounding below claims ONLY the `domain_topic`/`has_domain_topic`
//!    axis — the one the loaded corpus actually witnesses. The generated
//!    real-corpus test still sweeps the genre subtree for `exemplifies`
//!    edges (should OEWN gain some in a future release), honestly reporting
//!    whatever count the loaded data has, rather than asserting a
//!    fabricated one.
//!
//! # Literature
//!
//! - **Miller (1995)** *WordNet: A Lexical Database for English*, CACM
//!   38(11) — the general WordNet source.
//! - **Bentivogli & Pianta (2004)** "Extending WordNet with Syntagmatic
//!   Information" *Proc. GWC 2004* — the `domain_topic`/`has_domain_topic`
//!   pointer pair this module grounds `Genre` against.
//! - **Magnini & Cavaglià (2000)** *Integrating Subject Field Codes into
//!   WordNet*, Proc. LREC 2000 — the domain-topic annotation methodology
//!   WordNet's pointers implement.
//! - **IFLA FRBR Study Group (1998)** — §3.4 Concept (Group 3), the FRBR
//!   entity Genre specializes.

#[allow(unused_imports)]
use alloc::{boxed::Box, vec, vec::Vec};

use pr4xis::ontology::Axiom;
use pr4xis_runtime::ontology::ConceptRef;

use super::ontology::FrbrConcept;
use crate::cognitive::linguistics::english::{
    domain_topic_relation_kind, has_domain_topic_relation_kind,
};

/// The WordNet relation-kind vocabulary a `FrbrConcept` grounds through --
/// a rich type (not a bare enum tag) carrying BOTH directions of the
/// domain-topic pointer pair a Genre instance actually witnesses in the
/// loaded corpus (Bentivogli & Pianta 2004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenreWordNetGrounding {
    /// The loaded relation kind a DOMAIN synset (e.g. "law", "literature")
    /// carries to each of its member concepts.
    pub domain_to_member: ConceptRef,
    /// The inverse: the loaded relation kind a MEMBER synset carries back
    /// to its domain.
    pub member_to_domain: ConceptRef,
}

/// The grounding classifier: `Genre -> Some(GenreWordNetGrounding)` (the
/// `domain_topic`/`has_domain_topic` pointer pair, real and populated in
/// the loaded corpus -- 6433 edges corpus-wide, 19 directly under
/// "literature"); every other `FrbrConcept` honestly `None` -- Work,
/// Expression, and Manifestation are FRBR Group 1 "products of
/// intellectual endeavour" (IFLA FRBR 1998 §3.2), with no WordNet
/// synset-relation counterpart at all.
pub fn wordnet_grounding_of_frbr(concept: FrbrConcept) -> Option<GenreWordNetGrounding> {
    match concept {
        FrbrConcept::Genre => Some(GenreWordNetGrounding {
            domain_to_member: has_domain_topic_relation_kind(),
            member_to_domain: domain_topic_relation_kind(),
        }),
        FrbrConcept::Work | FrbrConcept::Expression | FrbrConcept::Manifestation => None,
    }
}

/// The keystone axiom: `Genre` is grounded in the loaded WordNet
/// domain-topic relation-kind pair, proven present rather than assumed --
/// the FRBR counterpart of `formal::mereology::wordnet_grounding`'s
/// `ProperPartAndWholeAreGroundedInWordNet`.
pub struct GenreGroundsInWordNetDomainTopic;

impl Axiom for GenreGroundsInWordNetDomainTopic {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let genre_grounded = wordnet_grounding_of_frbr(FrbrConcept::Genre)
            == Some(GenreWordNetGrounding {
                domain_to_member: has_domain_topic_relation_kind(),
                member_to_domain: domain_topic_relation_kind(),
            });
        let others_ungrounded = wordnet_grounding_of_frbr(FrbrConcept::Work).is_none()
            && wordnet_grounding_of_frbr(FrbrConcept::Expression).is_none()
            && wordnet_grounding_of_frbr(FrbrConcept::Manifestation).is_none();
        if genre_grounded && others_ungrounded {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GenreGroundsInWordNetDomainTopic",
        "FrbrConcept::Genre grounds in WordNet's has_domain_topic/domain_topic relation-kind pair; Work/Expression/Manifestation have no WordNet synset-relation counterpart",
        "Bentivogli & Pianta (2004) Proc. GWC 2004; IFLA FRBR Study Group (1998) \u{00a7}3.4"
    );
}

pr4xis::register_axiom!(
    GenreGroundsInWordNetDomainTopic,
    "Bentivogli & Pianta (2004); IFLA FRBR Study Group (1998) \u{00a7}3.4"
);

/// A representative-fixture composition check (honestly scoped exactly as
/// the sibling A4 mereology/counting grounding is: two small hand
/// fixtures, NOT the full corpus -- the full-corpus sweep is the separate
/// [`GenreDomainTopicAgreesAcrossLoadedEnglishWordNet`] axiom below).
/// CORRECTED direction (2026-07-21): the DOMAIN synset carries
/// `has_domain_topic` to its members; the MEMBER carries `domain_topic`
/// back to its domain -- matching the loaded corpus, not the reversed
/// assumption the original fixture made.
pub struct GenreDomainTopicRoundTripsOnFixtureWordNet;

impl Axiom for GenreDomainTopicRoundTripsOnFixtureWordNet {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use crate::cognitive::linguistics::english::English;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // Fixture 1: "literature" (domain) has_domain_topic "epic" (member);
        // "epic" domain_topic "literature" -- mirrors the real
        // oewn-06376048-n/oewn-06391344-n shape (a genre term topically
        // scoped under the literature domain).
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="frbr-fixture" label="FRBR" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-epic-n"><Lemma writtenForm="epic" partOfSpeech="n"/><Sense id="epic-n-01" synset="s-epic"/></LexicalEntry>
    <LexicalEntry id="e-literature-n"><Lemma writtenForm="literature" partOfSpeech="n"/><Sense id="literature-n-01" synset="s-literature"/></LexicalEntry>
    <Synset id="s-epic" ili="i1" partOfSpeech="n" members="e-epic-n">
      <Definition>a long narrative poem telling of a hero's deeds</Definition>
      <SynsetRelation relType="domain_topic" target="s-literature"/>
    </Synset>
    <Synset id="s-literature" ili="i2" partOfSpeech="n" members="e-literature-n">
      <Definition>creative writing of recognized artistic value</Definition>
      <SynsetRelation relType="has_domain_topic" target="s-epic"/>
    </Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);
        let Some(&epic_id) = english.lookup("epic").first() else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let Some(&literature_id) = english.lookup("literature").first() else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let fixture1_ok = english.has_domain_topic(literature_id).contains(&epic_id)
            && english.domain_topic(epic_id).contains(&literature_id);

        // Fixture 2: a differently-shaped example -- "law" (domain)
        // has_domain_topic "patent" (member), broadening the evidence past
        // one hand-picked pair (mirrors the real oewn-08458195-n/
        // oewn-06563618-n shape this module's doc comment cites).
        let xml2 = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="frbr-fixture-2" label="FRBR2" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-patent-n"><Lemma writtenForm="patent" partOfSpeech="n"/><Sense id="patent-n-01" synset="s-patent"/></LexicalEntry>
    <LexicalEntry id="e-law-n"><Lemma writtenForm="law" partOfSpeech="n"/><Sense id="law-n-01" synset="s-law"/></LexicalEntry>
    <Synset id="s-patent" ili="j1" partOfSpeech="n" members="e-patent-n">
      <Definition>an official document granting a right or privilege</Definition>
      <SynsetRelation relType="domain_topic" target="s-law"/>
    </Synset>
    <Synset id="s-law" ili="j2" partOfSpeech="n" members="e-law-n">
      <Definition>the collection of rules imposed by authority</Definition>
      <SynsetRelation relType="has_domain_topic" target="s-patent"/>
    </Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn2) = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml2) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english2 = English::from_wordnet(&wn2);
        let Some(&patent_id) = english2.lookup("patent").first() else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let Some(&law_id) = english2.lookup("law").first() else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let fixture2_ok = english2.has_domain_topic(law_id).contains(&patent_id)
            && english2.domain_topic(patent_id).contains(&law_id);

        if fixture1_ok && fixture2_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GenreDomainTopicRoundTripsOnFixtureWordNet",
        "checked against two representative fixtures (literature/epic, law/patent): the DOMAIN synset's has_domain_topic edge and the MEMBER synset's domain_topic edge round-trip, both directions -- not proven over the full corpus (see GenreDomainTopicAgreesAcrossLoadedEnglishWordNet for that)",
        "Bentivogli & Pianta (2004) Proc. GWC 2004; Magnini & Cavagli\u{00e0} (2000) Proc. LREC 2000"
    );
}

pr4xis::register_axiom!(
    GenreDomainTopicRoundTripsOnFixtureWordNet,
    "Bentivogli & Pianta (2004); Magnini & Cavagli\u{00e0} (2000)"
);

/// Every `ConceptId` reachable from `roots` by following `children()`
/// (hyponymy) transitively -- a cycle-safe BFS over the loaded taxonomy
/// (WordNet's hypernym/hyponym DAG has no cycles in practice, but a
/// visited-set keeps this honestly bounded regardless).
fn taxonomy_descendants(
    english: &crate::cognitive::linguistics::english::English,
    roots: &[crate::cognitive::linguistics::english::ConceptId],
) -> Vec<crate::cognitive::linguistics::english::ConceptId> {
    use hashbrown::HashSet;
    let mut visited: HashSet<crate::cognitive::linguistics::english::ConceptId> = HashSet::new();
    let mut queue: Vec<crate::cognitive::linguistics::english::ConceptId> = roots.to_vec();
    let mut out = Vec::new();
    while let Some(id) = queue.pop() {
        for &child in english.children(id) {
            if visited.insert(child) {
                out.push(child);
                queue.push(child);
            }
        }
    }
    out
}

/// The GENERATED test proper (B2 spec): sweeps the REAL loaded English
/// WordNet corpus (`english_loaded()`, no `.take(N)`, no hand-picked
/// list) for
///
/// 1. every real `has_domain_topic` edge under every loaded sense of
///    "literature" -- for EACH member found, the inverse `domain_topic`
///    edge must hold; and
/// 2. every real `exemplifies` edge anywhere in the loaded genre subtree
///    (every taxonomy descendant of every loaded sense of "genre") -- for
///    EACH such edge found (honestly, currently zero; see this module's
///    top doc), the inverse `is_exemplified_by` edge must hold.
///
/// Both loop bounds come from the loaded data itself, not a constant --
/// see the `#[test]` below for the real counts this exercises.
pub struct GenreDomainTopicAgreesAcrossLoadedEnglishWordNet;

impl Axiom for GenreDomainTopicAgreesAcrossLoadedEnglishWordNet {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use crate::cognitive::linguistics::english::english_loaded;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        let (domain_topic_checked, domain_topic_agree, _exemplifies_checked, exemplifies_agree) =
            sweep_loaded_genre_grounding(english_loaded());

        // Non-trivial per Casati & Varzi-style honesty (mirrors the
        // mereology precedent's `non_trivial` check): the domain_topic
        // axis MUST have found real edges (it does today: 19 under
        // "literature" alone) or this axiom is vacuous. The exemplifies
        // axis is allowed to be zero -- see module doc.
        if domain_topic_checked > 0 && domain_topic_agree && exemplifies_agree {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GenreDomainTopicAgreesAcrossLoadedEnglishWordNet",
        "swept over EVERY real has_domain_topic edge under every loaded sense of literature, and every real exemplifies edge in the loaded genre taxonomy subtree -- the domain_topic/has_domain_topic pair round-trips for every real edge found (19+ today); the exemplifies pair is checked over whatever the loaded corpus has (currently zero -- OEWN 2025 models genre taxonomically, not via instance-of)",
        "Bentivogli & Pianta (2004) Proc. GWC 2004; Miller (1995) CACM 38(11)"
    );
}

pr4xis::register_axiom!(
    GenreDomainTopicAgreesAcrossLoadedEnglishWordNet,
    "Bentivogli & Pianta (2004); Miller (1995)"
);

/// Shared sweep logic (used by both the axiom above and the `#[test]`
/// that reports real counts): returns
/// `(domain_topic_edges_checked, domain_topic_all_agree,
///   exemplifies_edges_checked, exemplifies_all_agree)`.
fn sweep_loaded_genre_grounding(
    english: &crate::cognitive::linguistics::english::English,
) -> (usize, bool, usize, bool) {
    let literature_senses = english.lookup("literature");
    let mut domain_topic_checked = 0usize;
    let mut domain_topic_agree = true;
    for &domain in literature_senses {
        for &member in english.has_domain_topic(domain) {
            domain_topic_checked += 1;
            if !english.domain_topic(member).contains(&domain) {
                domain_topic_agree = false;
            }
        }
    }

    let genre_senses = english.lookup("genre");
    let genre_descendants = taxonomy_descendants(english, genre_senses);
    let mut exemplifies_checked = 0usize;
    let mut exemplifies_agree = true;
    for &g in genre_senses.iter().chain(genre_descendants.iter()) {
        for &class in english.exemplifies(g) {
            exemplifies_checked += 1;
            if !english.is_exemplified_by(class).contains(&g) {
                exemplifies_agree = false;
            }
        }
    }

    (
        domain_topic_checked,
        domain_topic_agree,
        exemplifies_checked,
        exemplifies_agree,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn genre_grounds_in_domain_topic_pair() {
        assert_eq!(
            wordnet_grounding_of_frbr(FrbrConcept::Genre),
            Some(GenreWordNetGrounding {
                domain_to_member: has_domain_topic_relation_kind(),
                member_to_domain: domain_topic_relation_kind(),
            })
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn ungrounded_frbr_concepts_stay_ungrounded() {
        // No invented correspondences: Work/Expression/Manifestation have
        // no WordNet synset-relation counterpart, and this classifier says
        // so honestly rather than forcing a total mapping.
        use pr4xis::category::FinitelyGenerated;
        let grounded_count = FrbrConcept::variants()
            .into_iter()
            .filter(|c| wordnet_grounding_of_frbr(*c).is_some())
            .count();
        assert_eq!(grounded_count, 1, "exactly Genre should ground in WordNet");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn keystone_axiom_holds() {
        assert!(GenreGroundsInWordNetDomainTopic.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fixture_composition_holds() {
        assert!(GenreDomainTopicRoundTripsOnFixtureWordNet.verify().is_ok());
    }

    /// THE generated test (B2 spec item 2): walks the REAL loaded English
    /// WordNet corpus and reports the real edge counts it exercised -- run
    /// with `--nocapture` to see them. Not a fixed count, not a hardcoded
    /// list: whatever `english_loaded()` actually has today.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn genre_domain_topic_agrees_across_loaded_english_wordnet() {
        use crate::cognitive::linguistics::english::english_loaded;
        let (domain_topic_checked, domain_topic_agree, exemplifies_checked, exemplifies_agree) =
            sweep_loaded_genre_grounding(english_loaded());
        eprintln!(
            "GenreDomainTopicAgreesAcrossLoadedEnglishWordNet: swept {domain_topic_checked} \
             real has_domain_topic(literature) edges (all agree: {domain_topic_agree}), \
             {exemplifies_checked} real exemplifies edges in the genre taxonomy subtree \
             (all agree: {exemplifies_agree})"
        );
        assert!(
            domain_topic_checked > 0,
            "the loaded corpus must have at least one real has_domain_topic(literature) edge"
        );
        assert!(
            domain_topic_agree,
            "every swept domain_topic pair must round-trip"
        );
        assert!(
            exemplifies_agree,
            "every swept exemplifies pair must round-trip"
        );
        assert!(
            GenreDomainTopicAgreesAcrossLoadedEnglishWordNet
                .verify()
                .is_ok()
        );
    }
}
