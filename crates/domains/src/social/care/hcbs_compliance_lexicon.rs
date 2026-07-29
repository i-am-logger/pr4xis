//! HCBS Workforce / Compliance Lexicon (`hcbs_compliance_lexicon@2026`) —
//! runtime loader.
//!
//! Wraps the WN-LMF bundle at
//! `crates/domains/data/care/hcbs_compliance_lexicon.xml` (registered in
//! `praxis.toml` as `hcbs_compliance_lexicon@2026`, kind
//! `HcbsComplianceLexicon`, pinned in `praxis.lock`) as a chat-composable
//! [`RuntimeOntology`](pr4xis_runtime::ontology::RuntimeOntology) — the
//! Track 2 sibling of
//! [`caregiving_lexicon`](super::caregiving_lexicon): one praxis `Concept`
//! per compliance concept, ONE `Definition` whose authorities ride the
//! SEPARATE `dcterms:source` provenance channel (never the gloss text),
//! `canonicalForm` lexicalization for every surface
//! (canonical lemma and synonyms: "evv", "80/20 rule", "settings rule"),
//! and the verified isA taxonomy as `Subsumption` edges (targeted case
//! management → case management services, prevocational services →
//! habilitation services, clean claim → claim, ...).
//!
//! # Literature
//!
//! Per the bundled XML's header — each synset's own authorities are carried
//! STRUCTURALLY, in its WN-LMF `dc:source` attribute, and reach a reader
//! through the
//! [`lexicon_provenance`](crate::applied::data_provisioning::lexicon_provenance)
//! channel rather than as prose inside the gloss. The high-level sources are:
//!
//! - **Loaded U.S. Code Title 42** (`usc_title_42@pl-119-90`) — the EVV
//!   statutory definitions and FMAP-reduction mandate, 42 USC 1396b(l)
//!   (section 12006 of the 21st Century Cures Act, Pub. L. 114-255),
//!   verified against the on-disk corpus XML; the shared Track 1/Track 2
//!   statutory terms (respite care, habilitation services, case
//!   management services, the 1915(c) waiver authority).
//! - **42 CFR** 440, 441, 447, 438, 455; **89 FR 40542** (May 10, 2024,
//!   CMS-2442-F, the Ensuring Access to Medicaid Services final rule) —
//!   HCBS waiver / person-centered planning / settings / payment
//!   adequacy / incident management / billing / program integrity.
//! - **31 USC 3729** — the False Claims Act (Title 31 is not a loaded
//!   USC title, hence carried here); **42 CFR 400.203** (FFP);
//!   **HealthCare.gov Glossary** (prior authorization).

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

/// The registered source name (`praxis.toml`
/// `[sources.hcbs_compliance_lexicon]`).
pub const HCBS_COMPLIANCE_LEXICON_NAME: &str = "hcbs_compliance_lexicon";

/// The registered source version (`hcbs_compliance_lexicon@2026`, pinned
/// in `praxis.lock`; equals the bundle's `<Lexicon version>`).
pub const HCBS_COMPLIANCE_LEXICON_VERSION: &str = "2026";

/// The committed `.prx` — the content-addressed envelope carrying the
/// authored lexicon; the raw `.xml` is git-tracked source-of-truth,
/// excluded from the published crate (the `us_legal_lexicon` shape).
/// `pub` so the registry-driven residency dispatch
/// ([`chat_lexicons`](crate::applied::data_provisioning::chat_lexicons))
/// can map this source's taxonomy kind to its committed bundle.
pub const HCBS_COMPLIANCE_LEXICON_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/care/hcbs_compliance_lexicon.prx"
));

/// The registered `hcbs_compliance_lexicon@2026` source as a
/// chat-composable
/// [`RuntimeOntology`](pr4xis_runtime::ontology::RuntimeOntology) — the
/// generic WN-LMF lexicon bridge
/// ([`lexicon_runtime_ontology`](crate::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology))
/// parameterized by THIS source's registered `(name, version, committed
/// .prx)`. No lexicon-specific projection code.
#[cfg(feature = "std")]
pub fn hcbs_compliance_lexicon_runtime_ontology() -> Result<
    pr4xis_runtime::ontology::RuntimeOntology,
    crate::cognitive::linguistics::english::bridge::LexiconBridgeError,
> {
    crate::applied::data_provisioning::lexicon_provenance::lexicon_runtime_ontology(
        HCBS_COMPLIANCE_LEXICON_NAME,
        HCBS_COMPLIANCE_LEXICON_VERSION,
        HCBS_COMPLIANCE_LEXICON_PRX,
    )
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::english::bridge::{FORM_KIND, project_lexicon_archive};
    use pr4xis_runtime::archive::Archive;
    use pr4xis_runtime::definition::SOURCE_KIND;

    /// The REAL registered lexicon, loaded once and projected once (the
    /// `us_legal_lexicon` test shape).
    fn lexicon_projection() -> &'static (English, Archive) {
        static PROJ: OnceLock<(English, Archive)> = OnceLock::new();
        PROJ.get_or_init(|| {
            let xml = crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded(
                HCBS_COMPLIANCE_LEXICON_NAME,
                HCBS_COMPLIANCE_LEXICON_VERSION,
                HCBS_COMPLIANCE_LEXICON_PRX,
            );
            let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(&xml)
                .expect("the registered hcbs_compliance_lexicon parses as WN-LMF");
            let english = English::from_wordnet(&wn);
            let archive = project_lexicon_archive(&english);
            (english, archive)
        })
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_real_lexicon_projection_is_referentially_closed() {
        // Every projected edge — canonicalForm lexicalization AND the
        // hypernym taxonomy — names a DECLARED node (the closure
        // `materialize` later enforces, asserted at the projection).
        let (_, archive) = lexicon_projection();
        let declared: alloc::collections::BTreeSet<&str> =
            archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &archive.nodes {
            for (kind, target) in &n.edges {
                let to = target
                    .local_name()
                    .expect("a projected lexicon edge is a same-archive local edge");
                assert!(
                    declared.contains(to),
                    "edge {}--{kind}-->{to} names an undeclared node",
                    n.name
                );
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_written_form_resolves_and_mints_exactly_one_form_atom() {
        let (english, archive) = lexicon_projection();
        let mut count = 0usize;
        for w in english.word_index.words() {
            assert!(
                !english.lookup(w).is_empty(),
                "written form {w:?} must resolve to at least one concept"
            );
            let atoms = archive
                .nodes
                .iter()
                .filter(|n| n.kind == FORM_KIND && n.name == w)
                .count();
            assert_eq!(
                atoms, 1,
                "written form {w:?} must mint exactly one Form atom"
            );
            count += 1;
        }
        assert!(
            count >= 80,
            "the authored lexicon carries 100 surfaces; loaded only {count}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_registered_lexicon_materializes_as_a_runtime_ontology() {
        let onto = hcbs_compliance_lexicon_runtime_ontology()
            .expect("the registered hcbs_compliance_lexicon materializes");
        assert_eq!(
            onto.id().as_str(),
            HCBS_COMPLIANCE_LEXICON_NAME,
            "the ontology is named after its registered source"
        );
        let archive = onto
            .to_owned_archive()
            .expect("the materialized archive round-trips");
        // The english_functor relabeled every raw Synset generator into the
        // praxis Concept kind; the two other kinds are the ontolex:Form surface
        // atom and the dcterms:BibliographicResource citation atom the
        // provenance join appends.
        for n in &archive.nodes {
            assert!(
                n.kind == "Concept" || n.kind == FORM_KIND || n.kind == SOURCE_KIND,
                "unexpected node kind {:?} on {:?}",
                n.kind,
                n.name
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn a_compliance_terms_written_form_defines_through_the_composed_reasoner() {
        use crate::cognitive::linguistics::composed::{ComposedReasoner, GroundedConcept};
        use crate::cognitive::linguistics::english::LexicalReasoner;
        use alloc::rc::Rc;

        let onto = hcbs_compliance_lexicon_runtime_ontology()
            .expect("the registered hcbs_compliance_lexicon materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        // The define-style probe: canonical lemmas AND synonym/abbreviation
        // surfaces resolve to the loaded concept. The gloss is the text a
        // direct-care worker is recited; its authority reaches the same reader
        // through the SEPARATE `definition_sources` channel — asserted in both
        // directions so the two can never be confused for one another.
        for (surface, cited_in) in [
            (
                "electronic visit verification system",
                "42 USC 1396b(l)(5)(A)",
            ),
            // abbreviation surface → SAME synset, SAME cited gloss
            ("evv", "42 USC 1396b(l)(5)(A)"),
            ("80/20 rule", "42 CFR 441.302(k)"),
            ("critical incident", "42 CFR 441.302(a)(6)(i)(A)"),
            ("fraud", "42 CFR 455.2"),
        ] {
            let loaded = composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                .unwrap_or_else(|| panic!("{surface:?} must resolve to a loaded lexicon concept"));
            let concept = composed
                .concept(loaded)
                .expect("the loaded concept's view resolves");
            let gloss = concept
                .definitions()
                .next()
                .unwrap_or_else(|| panic!("{surface:?}'s definition must be reachable"));
            let sources: alloc::vec::Vec<&str> = concept.definition_sources().iter().collect();
            assert!(
                sources.contains(&cited_in),
                "{surface:?} must SURFACE its authority {cited_in:?} through the \
                 provenance channel; got {sources:?}"
            );
            assert!(
                !gloss.contains(cited_in),
                "{surface:?}'s recited gloss must NOT carry the citation as prose; \
                 got {gloss:?}"
            );
        }

        // The other direction: a WordNet substrate concept — one this lexicon
        // does not define — surfaces NO provenance, so a definition without a
        // declared source can never gain a spurious one.
        let substrate = composed
            .lookup("dog")
            .iter()
            .copied()
            .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::English(_))))
            .expect("the English substrate resolves its own word");
        assert!(
            composed
                .concept(substrate)
                .expect("the substrate concept's view resolves")
                .definition_sources()
                .is_empty(),
            "a WordNet gloss cites no document and must claim none"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_verified_taxonomy_answers_is_a_through_loaded_parents() {
        use crate::cognitive::linguistics::composed::{ComposedReasoner, GroundedConcept};
        use crate::cognitive::linguistics::english::LexicalReasoner;
        use alloc::rc::Rc;

        let onto = hcbs_compliance_lexicon_runtime_ontology()
            .expect("the registered hcbs_compliance_lexicon materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        let loaded_id = |surface: &str| {
            composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                .unwrap_or_else(|| panic!("{surface:?} must resolve to a loaded lexicon concept"))
        };

        // The verified isA edges: the waiver-service and billing taxonomies
        // answer "is X a Y" from loaded parents.
        for (child, parent) in [
            ("targeted case management", "case management services"),
            ("prevocational services", "habilitation services"),
            ("supported employment services", "habilitation services"),
            ("clean claim", "claim"),
            ("respite care", "home and community-based services"),
        ] {
            let c = loaded_id(child);
            let p = loaded_id(parent);
            assert!(
                composed.parents(c).contains(&p),
                "{child:?} must have {parent:?} among its loaded parents"
            );
        }
    }
}
