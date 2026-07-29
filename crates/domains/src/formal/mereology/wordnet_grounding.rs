//! The turing-benchmark A4 keystone: grounding `MereologyTheory`'s abstract
//! `ProperPart`/`Whole` in WordNet's concrete `Meronym`/`Holonym` -- the
//! bridge that lets a loaded `mero_part`/`holo_part` edge in the English
//! lexicon be read as a witness of the CEM theory in `ontology.rs`, not just
//! a same-named coincidence.
//!
//! This is deliberately NOT a `pr4xis::category::Functor` impl: a genuine
//! functor's `map_object` is total over the whole source category, and only
//! two of `MereologyTheory`'s thirteen concepts (`ProperPart`, `Whole`) have
//! an honestly-citable WordNet counterpart -- `Overlap`/`Fusion`/`Atom`/
//! `Gunk`/etc. have none. Forcing a total mapping would mean inventing nine
//! correspondences no source grounds. The partial, explicitly-`Option`
//! classifier below asserts only what it can cite (Miller 1995; Casati &
//! Varzi 1999).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::Axiom;

use super::ontology::MereologyTheoryConcept;
use crate::cognitive::linguistics::wordnet::ontology::WordNetConcept;

/// The grounding classifier: `ProperPart -> Meronym`, `Whole -> Holonym`,
/// every other `MereologyTheoryConcept` honestly `None` (no loaded WordNet
/// counterpart). Casati & Varzi (1999) \u{00a7}3.1 defines `ProperPart` as the
/// smaller-or-equal, non-identical term of a parthood relation -- exactly
/// Miller (1995)'s `Meronym` ("a part-of relation: wheel is a meronym of
/// car"); `Whole` is Casati & Varzi's larger term, the fusion of its
/// parts -- exactly Miller's `Holonym` (the whole-of inverse of `Meronym`).
pub fn wordnet_concept_of_mereology(concept: MereologyTheoryConcept) -> Option<WordNetConcept> {
    match concept {
        MereologyTheoryConcept::ProperPart => Some(WordNetConcept::Meronym),
        MereologyTheoryConcept::Whole => Some(WordNetConcept::Holonym),
        MereologyTheoryConcept::Part
        | MereologyTheoryConcept::Overlap
        | MereologyTheoryConcept::Underlap
        | MereologyTheoryConcept::Disjoint
        | MereologyTheoryConcept::Fusion
        | MereologyTheoryConcept::Sum
        | MereologyTheoryConcept::Product
        | MereologyTheoryConcept::Composition
        | MereologyTheoryConcept::Atom
        | MereologyTheoryConcept::Gunk
        | MereologyTheoryConcept::Supplementation => None,
    }
}

/// The keystone axiom: `ProperPart` and `Whole` are BOTH grounded in loaded
/// WordNet vocabulary -- the two concepts every downstream consumer (B1
/// qualitative physics' containment/support, B3 geography's RCC region-
/// parthood) actually needs, proven present rather than assumed.
pub struct ProperPartAndWholeAreGroundedInWordNet;

impl Axiom for ProperPartAndWholeAreGroundedInWordNet {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let proper_part_grounded = wordnet_concept_of_mereology(MereologyTheoryConcept::ProperPart)
            == Some(WordNetConcept::Meronym);
        let whole_grounded = wordnet_concept_of_mereology(MereologyTheoryConcept::Whole)
            == Some(WordNetConcept::Holonym);
        if proper_part_grounded && whole_grounded {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ProperPartAndWholeAreGroundedInWordNet",
        "MereologyTheory's ProperPart grounds in WordNet's Meronym and Whole grounds in Holonym -- the keystone that lets loaded mero_part/holo_part edges witness the CEM theory",
        "Casati & Varzi (1999) Parts and Places \u{00a7}3.1; Miller (1995) WordNet: A Lexical Database for English, CACM 38(11)"
    );
}

pr4xis::register_axiom!(
    ProperPartAndWholeAreGroundedInWordNet,
    "Casati & Varzi (1999) Parts and Places \u{00a7}3.1; Miller (1995) CACM 38(11)"
);

/// The A4 generated test proper (turing-benchmark spec). Honest scope
/// statement: the turing-benchmark spec frames this as "for every concept
/// with a non-empty loaded `parts()` list" -- this axiom's `verify()`
/// checks that property against representative fixtures (a 3-part and a
/// 1-part example, both below), NOT a proptest sweeping arbitrarily many
/// generated mereology trees and NOT the real WordNet corpus (no test in
/// this workspace yet exercises this composition against the loaded
/// `praxis-corpus-tests` WordNet fixture). Calling multiple hand-picked
/// examples "for every concept" would overclaim; what IS proven is that (a)
/// each part is REACHABLE via the loaded meronymy closure (the ProperPart
/// witness) and (b) the Counting ontology's
/// [`cardinality`](super::counting::cardinality) realization agrees with
/// the list length, for both fixtures -- the two sibling ontologies
/// (`MereologyTheory`+WordNet grounding, and `Counting`) compose correctly
/// on the data checked, not proven to compose correctly on all loaded data.
///
/// Separately honest: only the ProperPart/Meronym direction is exercised
/// against real loaded data here. The Whole/Holonym direction is asserted
/// structurally by [`ProperPartAndWholeAreGroundedInWordNet`] but has no
/// runtime counterpart to check it against -- `English` exposes `parts()`
/// (whole -> parts) but no reverse `holonym_of()` (part -> whole) accessor,
/// so there is currently no way to runtime-test the Holonym half the same
/// way. Building that accessor is out of scope for this fix.
pub struct MereologyPartsAgreeWithLoadedMeronymyAndCounting;

impl Axiom for MereologyPartsAgreeWithLoadedMeronymyAndCounting {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use crate::cognitive::linguistics::english::English;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // WordNet's own illustrative example (wordnet/ontology.rs Meronym
        // doc: "wheel is a meronym of car") -- a car composed of three
        // parts, none of them the loaded-inventory truncation this fixture
        // is deliberately larger than a `take(N)` cap would need to prove
        // (the runtime `parts()` accessor itself carries no cap; see
        // `english/ontology.rs:880-883`).
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="mereology-fixture" label="Mereology" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="entry-car-n"><Lemma writtenForm="car" partOfSpeech="n"/><Sense id="car-n-01" synset="synset-car-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-wheel-n"><Lemma writtenForm="wheel" partOfSpeech="n"/><Sense id="wheel-n-01" synset="synset-wheel-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-engine-n"><Lemma writtenForm="engine" partOfSpeech="n"/><Sense id="engine-n-01" synset="synset-engine-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-door-n"><Lemma writtenForm="door" partOfSpeech="n"/><Sense id="door-n-01" synset="synset-door-n-01"/></LexicalEntry>
    <Synset id="synset-car-n-01" ili="i1" partOfSpeech="n" members="entry-car-n">
      <Definition>a motor vehicle with four wheels</Definition>
      <SynsetRelation relType="mero_part" target="synset-wheel-n-01"/>
      <SynsetRelation relType="mero_part" target="synset-engine-n-01"/>
      <SynsetRelation relType="mero_part" target="synset-door-n-01"/>
    </Synset>
    <Synset id="synset-wheel-n-01" ili="i2" partOfSpeech="n" members="entry-wheel-n"><Definition>a circular frame that rotates on an axle</Definition></Synset>
    <Synset id="synset-engine-n-01" ili="i3" partOfSpeech="n" members="entry-engine-n"><Definition>the mechanism that converts fuel into mechanical energy</Definition></Synset>
    <Synset id="synset-door-n-01" ili="i4" partOfSpeech="n" members="entry-door-n"><Definition>a swinging or sliding barrier for an entrance</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn) = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english = English::from_wordnet(&wn);

        let Some(&car_id) = english.lookup("car").first() else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let parts = english.parts(car_id);

        // (a) each of the 3 loaded parts is reachable -- the ProperPart
        // witness (`Whole` grounds in `Holonym`, `ProperPart` in `Meronym`,
        // both proven above; here the RUNTIME closure agrees).
        let all_named = ["wheel", "engine", "door"].iter().all(|part_word| {
            english
                .lookup(part_word)
                .first()
                .is_some_and(|&id| parts.contains(&id))
        });

        // (b) Counting: the cardinality realization agrees with the
        // structural length -- never `.len()` trusted silently, always the
        // successor-counting function proven equal to it.
        let cardinality_agrees = super::counting::cardinality(parts).value == parts.len() as f64;
        let non_trivial = parts.len() == 3;

        // A SECOND, differently-shaped fixture (a 2-part bicycle, not a
        // 3-part car) -- broadening the evidence past one hand-picked
        // example, though still not a proptest/corpus sweep (see the
        // struct's honesty caveat above).
        let xml2 = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="mereology-fixture-2" label="Mereology2" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="entry-bicycle-n"><Lemma writtenForm="bicycle" partOfSpeech="n"/><Sense id="bicycle-n-01" synset="synset-bicycle-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-frame-n"><Lemma writtenForm="frame" partOfSpeech="n"/><Sense id="frame-n-01" synset="synset-frame-n-01"/></LexicalEntry>
    <LexicalEntry id="entry-saddle-n"><Lemma writtenForm="saddle" partOfSpeech="n"/><Sense id="saddle-n-01" synset="synset-saddle-n-01"/></LexicalEntry>
    <Synset id="synset-bicycle-n-01" ili="j1" partOfSpeech="n" members="entry-bicycle-n">
      <Definition>a two-wheeled vehicle pedaled by the rider</Definition>
      <SynsetRelation relType="mero_part" target="synset-frame-n-01"/>
      <SynsetRelation relType="mero_part" target="synset-saddle-n-01"/>
    </Synset>
    <Synset id="synset-frame-n-01" ili="j2" partOfSpeech="n" members="entry-frame-n"><Definition>the rigid supporting structure</Definition></Synset>
    <Synset id="synset-saddle-n-01" ili="j3" partOfSpeech="n" members="entry-saddle-n"><Definition>the seat a rider sits on</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let Ok(wn2) = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml2) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let english2 = English::from_wordnet(&wn2);
        let Some(&bicycle_id) = english2.lookup("bicycle").first() else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let parts2 = english2.parts(bicycle_id);
        let all_named2 = ["frame", "saddle"].iter().all(|part_word| {
            english2
                .lookup(part_word)
                .first()
                .is_some_and(|&id| parts2.contains(&id))
        });
        let cardinality_agrees2 = super::counting::cardinality(parts2).value == parts2.len() as f64;
        let non_trivial2 = parts2.len() == 2;

        if all_named
            && cardinality_agrees
            && non_trivial
            && all_named2
            && cardinality_agrees2
            && non_trivial2
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MereologyPartsAgreeWithLoadedMeronymyAndCounting",
        "checked against representative fixtures (a 3-part and a 2-part example): each loaded part is reachable via the meronymy closure and Counting::cardinality agrees with the list length -- not a proven universal over every loaded concept",
        "Casati & Varzi (1999) Parts and Places \u{00a7}3.1; Miller (1995) CACM 38(11); Frege (1884) Die Grundlagen der Arithmetik \u{00a7}68"
    );
}

pr4xis::register_axiom!(
    MereologyPartsAgreeWithLoadedMeronymyAndCounting,
    "Casati & Varzi (1999); Miller (1995); Frege (1884)"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn proper_part_grounds_in_meronym() {
        assert_eq!(
            wordnet_concept_of_mereology(MereologyTheoryConcept::ProperPart),
            Some(WordNetConcept::Meronym)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn whole_grounds_in_holonym() {
        assert_eq!(
            wordnet_concept_of_mereology(MereologyTheoryConcept::Whole),
            Some(WordNetConcept::Holonym)
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn ungrounded_mereology_concepts_stay_ungrounded() {
        // No invented correspondences: every OTHER MereologyTheory concept
        // has no loaded WordNet counterpart, and this classifier says so
        // honestly rather than forcing a total mapping.
        use pr4xis::category::FinitelyGenerated;
        let grounded_count = MereologyTheoryConcept::variants()
            .into_iter()
            .filter(|c| wordnet_concept_of_mereology(*c).is_some())
            .count();
        assert_eq!(
            grounded_count, 2,
            "exactly ProperPart and Whole should ground in WordNet"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn keystone_axiom_holds() {
        assert!(ProperPartAndWholeAreGroundedInWordNet.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mereology_parts_agree_with_loaded_meronymy_and_counting() {
        assert!(
            MereologyPartsAgreeWithLoadedMeronymyAndCounting
                .verify()
                .is_ok()
        );
    }
}
