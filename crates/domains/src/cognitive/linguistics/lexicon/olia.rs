#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::pos::PosTag;

// OLiA → PosTag functor.
//
// Maps OLiA (Ontologies of Linguistic Annotation) class IRIs to our PosTag
// category. This is a functor from the OLiA category to our lexical category.
//
// OLiA defines 1,300+ linguistic classes in OWL/DL. We map the ones relevant
// to morphosyntactic categories to our PosTag enum.
//
// Reference: Chiarcos & Sukhareva, OLiA (Semantic Web journal, 2015)
// OLiA reference model: http://purl.org/olia/olia.owl

const OLIA_NS: &str = "http://purl.org/olia/olia.owl#";

/// Map an OLiA class IRI to a PosTag.
/// This is the object map of the functor: OLiA → PosTag.
pub fn olia_to_pos(iri: &str) -> Option<PosTag> {
    let fragment = iri_fragment(iri)?;
    from_fragment(fragment)
}

/// The full OLiA Reference-Model IRI of a class fragment (e.g.
/// `InterrogativeAdverb` → `http://purl.org/olia/olia.owl#InterrogativeAdverb`).
/// The wire identity of a loaded OLiA class.
pub fn class_iri(fragment: &str) -> String {
    format!("{OLIA_NS}{fragment}")
}

/// The OLiA Reference Model itself, loaded from its **committed compact
/// `.prx.gz`** through the SINGLE generalized, registry-driven OWL-vocab load
/// mechanism ([`load_owl_vocabulary`][lov]) and cached for the process.
///
/// This is the LOADED grammatical-class vocabulary — `find` / `is_a` /
/// `subsumes` (`LoadedOwlVocabulary`) query it by IRI. A class identity is
/// "an OLiA Concept that resolves here", never a Rust enum mirroring the
/// ontology nor a bare string compared by `==`.
///
/// OLiA is loaded through the *same* path as every SPAR/PROV-O vocabulary — its
/// committed `data/ontologies/olia-2026-04-09.prx.gz`, admitted fail-closed
/// against the `praxis.lock` `[compact_archive_signatures]` pin. There is NO
/// raw-`.owl` fallback: the raw is fetch-only (`pr4xis update`) and ships in no
/// crate. A missing/unpinned committed `.prx.gz`, or one that fails the content
/// gate, panics with the fix to run, rather than silently degrading.
///
/// `prx`-gated: the committed-archive load needs the succinct decoder. On a
/// `std`-without-`prx` (or `no_std`/wasm-without-`prx`) build the coarse
/// exact-match [`from_fragment`] map is the only path (a tracked transitional
/// residue), so the loaded-model accessors below are `prx`-gated too.
///
/// [lov]: crate::social::software::markup::xml::owl::loaded_vocabularies::load_owl_vocabulary
#[cfg(feature = "prx")]
pub fn reference_model()
-> &'static crate::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary {
    use crate::applied::data_provisioning::registry::by_name;
    use crate::social::software::markup::xml::owl::loaded_vocabularies::load_owl_vocabulary_embedded;
    use crate::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
    use std::sync::OnceLock;

    // The committed compact `.prx.gz`, EMBEDDED with `include_bytes!` — the
    // wasm-safe load path (`reference_model` grounds the `ComposedReasoner` in
    // every build, wasm32 included, where there is no `std::fs`). The generalized
    // `load_owl_vocabulary` reads the `.prx.gz` from the registry/workspace path,
    // which a wasm build can't reach; embedding the bytes (as the pre-
    // generalization `olia::reference_model` did) keeps the SAME gated mechanism
    // (`load_owl_vocabulary_embedded` → `load_compact_prx_gz_gated` against the
    // `[compact_archive_signatures]` pin) without the filesystem. NO 1.2 MB OWL
    // re-parse, NO raw-`.owl` fallback.
    const OLIA_COMPACT_PRX_GZ: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/olia-2026-04-09.prx.gz"
    ));

    static MODEL: OnceLock<LoadedOwlVocabulary> = OnceLock::new();
    MODEL.get_or_init(|| {
        let entry = by_name("olia")
            .expect("the `olia` OntologyVocabulary must be registered in praxis.toml");
        load_owl_vocabulary_embedded("olia", &entry.version, OLIA_COMPACT_PRX_GZ).unwrap_or_else(
            |e| {
                panic!(
                    "olia::reference_model(): the embedded compact .prx.gz for `olia@{}` \
                     failed the [compact_archive_signatures] content gate: {e}",
                    entry.version
                )
            },
        )
    })
}

/// True iff `fragment` names a class that resolves in the loaded OLiA Reference
/// Model — the runtime grounding check that a class identity is a real loaded
/// OLiA Concept, not an arbitrary string.
#[cfg(feature = "prx")]
pub fn is_loaded_class(fragment: &str) -> bool {
    reference_model().find(&class_iri(fragment)).is_some()
}

/// Map an OLiA class fragment (the part after #) to a PosTag.
///
/// The single authority on which OLiA class fragments praxis recognizes;
/// the loaded `reference_model` is the ontology these fragments name.
pub fn from_fragment(fragment: &str) -> Option<PosTag> {
    // The top MorphosyntacticCategory subclasses → PosTag (OLiA Reference Model;
    // Chiarcos & Sukhareva 2015). The ONLY OLiA fragments named in Rust — every
    // one of OLiA's ~1300 subclasses (CommonNoun, InterrogativePronoun,
    // DefiniteArticle, …) resolves to the same PosTag by `rdfs:subClassOf`
    // closure over the loaded ontology, never by a hand-enumerated arm.
    //
    // Ordered MOST-SPECIFIC-FIRST: Copula / AuxiliaryVerb (⊑ Verb) before Verb,
    // Article (⊑ Determiner) before Determiner — else the general class would
    // shadow the specific PosTag.
    const BASE: &[(&str, PosTag)] = &[
        ("Copula", PosTag::Copula),
        ("AuxiliaryVerb", PosTag::Auxiliary),
        ("Article", PosTag::Article),
        ("Numeral", PosTag::Numeral),
        ("Verb", PosTag::Verb),
        ("Determiner", PosTag::Determiner),
        ("Pronoun", PosTag::Pronoun),
        ("Adjective", PosTag::Adjective),
        ("Adverb", PosTag::Adverb),
        ("Preposition", PosTag::Preposition),
        ("Conjunction", PosTag::Conjunction),
        ("Interjection", PosTag::Interjection),
        ("Particle", PosTag::Particle),
        ("Noun", PosTag::Noun),
    ];
    for (top, pos) in BASE {
        // `is_a` is false for self (subsumes returns false when child==parent),
        // so the exact-match check is required, not just the closure.
        if fragment == *top {
            return Some(*pos);
        }
        // The loaded-model subsumption closure needs the `prx`-gated committed
        // archive; without `prx` only the exact-match BASE table resolves (the
        // coarse fallback the `no_std`/wasm path uses).
        #[cfg(feature = "prx")]
        {
            if reference_model().is_a(&class_iri(fragment), &class_iri(top)) {
                return Some(*pos);
            }
        }
    }
    None
}

/// Extract the fragment (after #) from an IRI.
fn iri_fragment(iri: &str) -> Option<&str> {
    iri.rsplit_once('#').map(|(_, frag)| frag)
}

/// Check if an IRI belongs to the OLiA namespace.
pub fn is_olia_iri(iri: &str) -> bool {
    iri.starts_with(OLIA_NS)
}

// ── Cross-functor: SemanticEffect → OLiA ─────────────────────────────

/// Map a [`SemanticEffect`](super::super::morphology::SemanticEffect)
/// to the OLiA class fragments that realise it.
///
/// Cross-functor from praxis's morphology ontology into OLiA's
/// Morphosyntax sub-ontology. Lets downstream tooling reason about
/// each morphological rule's *grammatical category* through the
/// shared OLiA vocabulary rather than the praxis-internal enum
/// variant names.
///
/// Returns an empty `Vec` for effects with no direct OLiA mapping
/// (e.g. `SemanticEffect::PosChange` — POS changes are structural
/// edges, not OLiA feature classes; `SemanticEffect::Repetition`
/// — re-prefixing is derivational without a Morphosyntax slot).
///
/// Reference: Chiarcos & Sukhareva (2015), OLiA Morphosyntax model
/// at <http://purl.org/olia/olia.owl>.
pub fn semantic_effect_to_olia_fragments(
    effect: super::super::morphology::SemanticEffect,
) -> Vec<&'static str> {
    use super::super::morphology::SemanticEffect;
    match effect {
        // Morphological negation — closest OLiA class is the
        // operator-level `NegativeParticle` since OLiA's
        // Morphosyntax model doesn't carry a Negation feature
        // directly on the affixed word.
        SemanticEffect::Negation => vec!["NegativeParticle"],
        // No OLiA Morphosyntax class for derivational repetition.
        SemanticEffect::Repetition => vec![],
        // POS change is a category transition, not an OLiA feature.
        SemanticEffect::PosChange => vec![],
        // OLiA Number → {Singular, Plural, Dual, ...}. We emit the
        // category here; the rule's input vs output Number values
        // live separately on each instance.
        SemanticEffect::NumberChange => vec!["Number", "Singular", "Plural"],
        // OLiA Tense → {Past, Present, Future, ...}.
        SemanticEffect::TenseChange => vec!["Tense", "Past", "Present", "Future"],
        // OLiA Aspect with the Progressive class specifically.
        SemanticEffect::Progressive => vec!["Aspect", "ProgressiveAspect"],
        // OLiA covers agent-nominal derivation under
        // AgentiveNoun (in the Lexinfo extension; the base OLiA
        // does not carry it). Returned for downstream resolution.
        SemanticEffect::AgentNoun => vec!["AgentiveNoun"],
        // De-adjectival quality nouns: OLiA does not have a
        // direct class — `StativeVerb` is the closest sibling.
        // Empty until a richer derivational extension is loaded.
        SemanticEffect::QualityNoun => vec![],
    }
}

/// The canonical OLiA class fragment for a coarse [`PosTag`] — the
/// PosTag → OLiA-top-class anchor.
///
/// This is the irreducible closed-enum → canonical-OLiA-name bridge (the
/// inverse-direction sibling of [`from_fragment`]'s BASE anchors): it names the
/// top MorphosyntacticCategory class for each coarse tag and constructs no
/// knowledge. The prior `pos_to_olia_fragments` additionally HAND-LISTED a few
/// subclasses per tag (CommonNoun, ProperNoun, ModalVerb, …) — residue, since
/// the loaded OLiA `reference_model()` subsumption closure already determines
/// the descendants of each anchor, and production only ever used the anchor
/// (audit 2026-06-12 D-13).
pub fn canonical_olia_fragment(pos: PosTag) -> &'static str {
    match pos {
        PosTag::Noun => "Noun",
        PosTag::Verb => "Verb",
        PosTag::Copula => "Copula",
        PosTag::Auxiliary => "AuxiliaryVerb",
        PosTag::Determiner => "Determiner",
        PosTag::Article => "Article",
        PosTag::Adjective => "Adjective",
        PosTag::Adverb => "Adverb",
        PosTag::Pronoun => "Pronoun",
        PosTag::Preposition => "Preposition",
        PosTag::Conjunction => "Conjunction",
        PosTag::Interjection => "Interjection",
        PosTag::Particle => "Particle",
        PosTag::Numeral => "Numeral",
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::morphology::SemanticEffect;
    use super::*;

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn olia_iri_round_trip() {
        let iri = "http://purl.org/olia/olia.owl#Noun";
        assert!(is_olia_iri(iri));
        assert_eq!(olia_to_pos(iri), Some(PosTag::Noun));
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn canonical_olia_fragment_round_trips() {
        // The PosTag → OLiA-top-class anchor round-trips through `from_fragment`
        // (which resolves via the LOADED OLiA subsumption closure), so the anchor
        // is a real loaded class — no hand-listed subclasses needed.
        for pos in [
            PosTag::Noun,
            PosTag::Verb,
            PosTag::Adjective,
            PosTag::Adverb,
        ] {
            let frag = canonical_olia_fragment(pos);
            assert_eq!(
                from_fragment(frag),
                Some(pos),
                "anchor `{frag}` did not map back to {pos:?}"
            );
        }
    }

    // ── SemanticEffect → OLiA cross-functor ─────────────────────────

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn number_change_maps_to_olia_number_singular_plural() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::NumberChange);
        assert!(frags.contains(&"Number"));
        assert!(frags.contains(&"Singular"));
        assert!(frags.contains(&"Plural"));
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn tense_change_maps_to_olia_tense_axis() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::TenseChange);
        assert!(frags.contains(&"Tense"));
        assert!(frags.contains(&"Past"));
        assert!(frags.contains(&"Present"));
        assert!(frags.contains(&"Future"));
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn progressive_maps_to_olia_progressive_aspect() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::Progressive);
        assert!(frags.contains(&"Aspect"));
        assert!(frags.contains(&"ProgressiveAspect"));
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn negation_maps_to_olia_negative_particle() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::Negation);
        assert!(frags.contains(&"NegativeParticle"));
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn pos_change_has_no_olia_feature_mapping() {
        // PosChange is a category-edge, not an OLiA feature.
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::PosChange);
        assert!(frags.is_empty());
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn repetition_has_no_olia_feature_mapping() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::Repetition);
        assert!(frags.is_empty());
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn every_semantic_effect_handled() {
        // Coverage: every variant must be callable through the
        // cross-functor without panicking. Concept::variants() lists
        // the full set so adding a new variant forces a deliberate
        // mapping decision here.
        use pr4xis::category::FinitelyGenerated;
        for effect in SemanticEffect::variants() {
            let _ = semantic_effect_to_olia_fragments(effect);
        }
    }

    // ── Property-based laws for the OLiA cross-functor ─────────────

    use proptest::prelude::*;

    fn arb_semantic_effect() -> impl Strategy<Value = SemanticEffect> {
        prop_oneof![
            Just(SemanticEffect::Negation),
            Just(SemanticEffect::Repetition),
            Just(SemanticEffect::PosChange),
            Just(SemanticEffect::NumberChange),
            Just(SemanticEffect::TenseChange),
            Just(SemanticEffect::Progressive),
            Just(SemanticEffect::AgentNoun),
            Just(SemanticEffect::QualityNoun),
        ]
    }

    proptest! {
        #[test]
        fn property_olia_fragments_are_valid_class_names(
            effect in arb_semantic_effect(),
        ) {
            // Every emitted fragment must parse as a valid OLiA
            // class fragment via `from_fragment` — either resolving
            // to a PosTag (when the fragment is a POS class) or
            // being a documented OLiA Morphosyntax class. Empty
            // returns are valid for variants with no OLiA mapping.
            //
            // Documented OLiA Morphosyntax + structural fragments
            // we emit (per Chiarcos & Sukhareva 2015):
            const OLIA_MORPHOSYNTAX_CLASSES: &[&str] = &[
                "Number", "Singular", "Plural", "Dual",
                "Tense", "Past", "Present", "Future",
                "Aspect", "ProgressiveAspect", "PerfectiveAspect", "ImperfectiveAspect",
                "Mood", "Indicative", "Subjunctive", "Imperative",
                "Case", "Nominative", "Accusative", "Genitive", "Dative",
                "Person", "FirstPerson", "SecondPerson", "ThirdPerson",
                "Gender", "Masculine", "Feminine", "Neuter",
                "NegativeParticle", "AgentiveNoun", "StativeVerb",
            ];

            for fragment in semantic_effect_to_olia_fragments(effect) {
                let is_pos = from_fragment(fragment).is_some();
                let is_morphosyntax = OLIA_MORPHOSYNTAX_CLASSES.contains(&fragment);
                prop_assert!(
                    is_pos || is_morphosyntax,
                    "fragment `{fragment}` from {effect:?} is neither a known PosTag class nor a documented OLiA Morphosyntax class"
                );
            }
        }

        #[test]
        fn property_olia_fragments_deterministic(
            effect in arb_semantic_effect(),
        ) {
            let a = semantic_effect_to_olia_fragments(effect);
            let b = semantic_effect_to_olia_fragments(effect);
            prop_assert_eq!(a, b);
        }

        #[test]
        fn property_olia_fragments_are_unique(
            effect in arb_semantic_effect(),
        ) {
            // No effect should emit duplicate fragments.
            let frags = semantic_effect_to_olia_fragments(effect);
            let unique: alloc::collections::BTreeSet<&&str> = frags.iter().collect();
            prop_assert_eq!(unique.len(), frags.len());
        }
    }

    pr4xis::register_praxis_value!(property_olia_fragments_are_valid_class_names, Verifiable);
    pr4xis::register_praxis_value!(property_olia_fragments_deterministic, Deterministic);
    pr4xis::register_praxis_value!(property_olia_fragments_are_unique, Verifiable);
}

#[cfg(all(test, feature = "prx"))]
mod prx_fast_load {
    use super::*;

    /// The FETCHED raw OLiA `.owl` bytes, read from disk via the registry's
    /// `local_path()` — NOT `include_str!`-embedded. The raw is fetch-only
    /// (`pr4xis update`) and ships in no crate; an absent raw is a hard FAILURE
    /// naming the fix, never a silent skip (the staleness guard cannot
    /// degrade-skip).
    fn fetched_olia_owl() -> std::string::String {
        use crate::applied::data_provisioning::registry::by_name;
        let entry = by_name("olia").expect("olia registered");
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .expect("workspace root");
        let path = root.join(entry.local_path());
        std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "OLiA raw .owl is not on disk at {} ({e}) — it is fetch-only; run \
                 `pr4xis update` to regenerate it before the staleness guard",
                path.display()
            )
        })
    }

    /// Regenerate the committed compact `.prx.gz` from the FETCHED authoritative
    /// OWL. `#[ignore]` — run manually (`cargo test -p pr4xis-domains -- --ignored
    /// regenerate_olia_compact_prx`) when the OWL changes; the committed
    /// artifact is what `reference_model` loads at runtime.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore]
    fn regenerate_olia_compact_prx() {
        use crate::social::software::markup::xml::owl::prx::emit_compact_owl_prx_gz;
        let owl = fetched_olia_owl();
        let prx_gz = emit_compact_owl_prx_gz(
            owl.as_bytes(),
            "olia",
            "2026-04-09",
            "http://purl.org/olia/olia.owl",
        )
        .expect("emit compact OLiA .prx.gz");
        let out = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/olia-2026-04-09.prx.gz"
        );
        std::fs::write(out, &prx_gz).expect("write committed OLiA .prx.gz");
    }

    /// The runtime loads OLiA and resolves the interrogative classes the
    /// OLiA→CCG functor keys on (fast path once the `.prx.gz` is wired in).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn resolves_the_interrogative_classes() {
        for fragment in [
            "InterrogativePronoun",
            "InterrogativeAdverb",
            "InterrogativeDeterminer",
        ] {
            assert!(
                is_loaded_class(fragment),
                "{fragment} must resolve in the loaded OLiA Reference Model"
            );
        }
    }

    /// STALENESS GUARD: the committed compact `.prx.gz` is faithful to the
    /// authoritative OWL — the class set the fast path loads equals what parsing
    /// the FETCHED raw `.owl` yields, so a stale artifact (OWL changed, `.prx.gz`
    /// not regenerated) is caught. Verifies WITHOUT a shipped raw: the raw is
    /// read from disk (fetched via `pr4xis update`); its absence FAILS in
    /// `fetched_olia_owl`, never skips.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bundled_prx_matches_the_owl() {
        use crate::social::software::markup::xml::owl::reader::read_owl;
        use crate::social::software::markup::xml::owl::vocabulary::LoadedOwlVocabulary;
        let owl = fetched_olia_owl();
        let from_owl = LoadedOwlVocabulary::from_owl_ontology(&read_owl(&owl).expect("parse OWL"));
        assert_eq!(
            reference_model().entity_count(),
            from_owl.entity_count(),
            "committed OLiA .prx.gz is stale — regenerate with `--ignored regenerate_olia_compact_prx`"
        );
    }
}

#[cfg(all(test, feature = "prx"))]
mod from_fragment_subsumption {
    use super::*;

    /// `from_fragment` derives the PosTag from the loaded OWL subsumption
    /// hierarchy: top classes match exactly, and ~1300 subclasses resolve by
    /// `rdfs:subClassOf` closure — none enumerated in Rust. Ordering (Copula/
    /// AuxiliaryVerb before Verb, Article before Determiner) must hold.
    #[pr4xis::praxis_value(Verifiable, Extensible, Honest)]
    #[test]
    fn subclasses_resolve_by_owl_closure() {
        // Top classes (exact match, no OWL needed).
        for (frag, pos) in [
            ("Noun", PosTag::Noun),
            ("Verb", PosTag::Verb),
            ("Copula", PosTag::Copula),
            ("AuxiliaryVerb", PosTag::Auxiliary),
            ("Determiner", PosTag::Determiner),
            ("Article", PosTag::Article),
            ("Pronoun", PosTag::Pronoun),
            ("Adjective", PosTag::Adjective),
            ("Adverb", PosTag::Adverb),
            ("Numeral", PosTag::Numeral),
        ] {
            assert_eq!(from_fragment(frag), Some(pos), "top class {frag}");
        }
        // Subclasses derived by subClassOf closure — NOT in the Rust base map.
        for (frag, pos) in [
            ("CommonNoun", PosTag::Noun),
            ("ProperNoun", PosTag::Noun),
            ("InterrogativePronoun", PosTag::Pronoun),
            ("InterrogativeAdverb", PosTag::Adverb),
            ("InterrogativeDeterminer", PosTag::Determiner),
            ("DefiniteArticle", PosTag::Article),
            ("ModalVerb", PosTag::Auxiliary),
        ] {
            assert_eq!(
                from_fragment(frag),
                Some(pos),
                "subclass {frag} via closure"
            );
        }
        // Ordering: a Copula must NOT resolve to Verb even though Copula ⊑ Verb.
        assert_eq!(from_fragment("Copula"), Some(PosTag::Copula));
        // Unknown fragment fails closed.
        assert_eq!(from_fragment("NotAnOliaClass"), None);
    }
}
