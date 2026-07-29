//! The concrete, first SSSOM producer: SUMO's WordNet↔SUMO EQUIVALENCE rows
//! ([`SumoRelationKind::Equivalence`]) minted as ONE [`SssomMappingSet`] of
//! `skos:exactMatch` correspondences between real OEWN synset URIs and SUMO
//! term URIs — the pattern every grounding lens in this codebase follows
//! (denotes/cites/defines): ONE concrete case built and proven before any
//! generalization, never a "mint an SSSOM set between any two peer archives"
//! mechanism.
//!
//! ## Scope: EQ rows only, ambiguous concepts excluded
//!
//! Only [`SumoRelationKind::Equivalence`] rows are "equivalent in meaning" per
//! the source's own legend (`super::ontology`'s module doc quotes it
//! verbatim) — `SUB`/`INST` are subsumption/instance-of, a DIFFERENT
//! correspondence relation SSSOM's `skos:exactMatch` cannot honestly carry,
//! and the three `Complement*` codes assert a NON-relation. A concept with
//! TWO OR MORE DISTINCT EQ terms is genuinely ambiguous (the source data
//! itself does not pick one) — [`ambiguous_eq_concepts`] DERIVES this set
//! from the loaded data on every call (never a hardcoded literal list; see
//! its own doc and the `excludes_all_five_known_ambiguous_concepts` test),
//! and [`sumo_eq_sssom_mapping_set`] excludes them: a `1:1 sameAs` mapping
//! from an ambiguous source row would be a fabricated precision claim the
//! data does not support.
//!
//! ## URI honesty
//!
//! - `subject_id` — `https://en-word.net/id/{oewn_synset_id}`. RE-VERIFIED
//!   2026-07-14 (build time): `https://en-word.net/id/oewn-01658725-s`
//!   returns HTTP 200, and the Open English WordNet's own published RDF
//!   (`https://en-word.net/static/english-wordnet-2025.ttl.gz`) declares
//!   `@prefix wnid: <https://en-word.net/id/>` as its OWN synset-resource
//!   namespace — so this is the project's real, machine-published URI
//!   scheme, not a guessed convention.
//! - `object_id` — `http://www.ontologyportal.org/SUMO.owl#{term}`. An
//!   earlier candidate, `http://www.adampease.org/OP/SUMO.owl#{term}`
//!   (the older third-party mirror commonly cited in SUMO/OWL literature —
//!   Adam Pease is SUMO's co-author), was RE-VERIFIED 2026-07-14 and found
//!   dead: `GET http://www.adampease.org/OP/SUMO.owl` returns HTTP 404
//!   (confirmed with a real `GET`, not just `HEAD` — the domain's behavior
//!   is inconsistent across request shapes, so a `HEAD`-only check would be
//!   misleading either way). SUMO's own primary repository
//!   (`ontologyportal/sumo`) ships only `.kif` sources under that path — no
//!   OWL translation — confirming the `adampease.org` copy was always a
//!   third-party conversion, never SUMO's own primary artifact.
//!   `http://www.ontologyportal.org/SUMO.owl.html` — SUMO's OWN official
//!   project site — was independently re-fetched (not merely assumed live
//!   from a prior report) and confirmed to serve the real translation: HTTP
//!   200, real `rdf:RDF`/`owl:Class` content. Its own `<rdfs:comment>`
//!   ontology header states, verbatim: "A provisional and necessarily lossy
//!   translation to OWL. Please see www.ontologyportal.org for the original
//!   KIF, which is the authoritative source." — disclosed here because it
//!   bears directly on how much a `skos:exactMatch` to a bare term string in
//!   this namespace should be trusted: the translation's own authors
//!   describe it as incomplete relative to the KIF original (OWL-DL's
//!   arity-≤2 restriction loses some of SUMO's higher-arity KIF axioms).
//!   This codebase has no locally-loaded SUMO OWL ontology to independently
//!   confirm a given TERM STRING still resolves within that live file, so
//!   `object_id`'s provenance here is "cited, currently-live third-party
//!   namespace, itself self-described as lossy", never "machine-verified
//!   against a loaded peer".
//!
//! ## Physical export
//!
//! [`sumo_eq_sssom_mapping_set`]'s output is serialized to the real, SSSOM
//! EMBEDDED-mode `.sssom.tsv` file
//! (`crates/domains/data/sumo/sumo-wordnet.sssom.tsv`) via
//! [`SssomMappingSet::to_sssom_tsv`], fed `SUMO_SSSOM_CURIE_MAP` (a
//! test-scoped constant — see this module's own test suite) — the
//! `skos:`/`semapv:` prefixes this producer's rows actually use. See the
//! `regenerate_sumo_wordnet_sssom_tsv` (`#[ignore]`d regen) and
//! `committed_sumo_wordnet_sssom_tsv_matches_source` (staleness guard) tests
//! there.
//!
//! # Literature
//!
//! - **Niles, I. & Pease, A. (2001)** "Towards a Standard Upper Ontology."
//!   *FOIS 2001*, pp. 2-9.
//! - **Niles, I. & Pease, A. (2003)** "Linking Lexicons and Ontologies:
//!   Mapping WordNet to the Suggested Upper Merged Ontology." *IEEE IKE
//!   2003*, pp. 412-416 — the hand-curated crosswalk this module reads,
//!   grounding [`MappingJustification::ManualMappingCuration`] (the paper's
//!   own title names this a LINKING/MAPPING exercise, not a lexical-matching
//!   algorithm).
//! - **Matentzoglu et al. (2022)** SSSOM, *Database* (Oxford) baac035 — see
//!   [`crate::formal::information::schema::sssom`] for the full citation.

#[allow(unused_imports)]
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::formal::information::schema::sssom::{
    MappingJustification, SssomMapping, SssomMappingSet,
};
use crate::formal::relations::ontology::equivalence_relation_kind;

use super::ontology::SumoRelationKind;

/// SSSOM's `predicate_id` for a SUMO EQ correspondence — SKOS `exactMatch`,
/// cross-checked (see `predicate_id_matches_the_registered_equivalence_relation_kind`)
/// against [`equivalence_relation_kind`]'s OWN definition text rather than
/// standing as a disconnected literal.
const EXACT_MATCH_PREDICATE_ID: &str = "skos:exactMatch";

/// SUMO's OWL term-namespace base — SUMO's own official project site
/// (`ontologyportal.org`), re-verified live 2026-07-14. See the module doc's
/// URI-honesty section for why this supersedes the older, now-dead
/// `adampease.org` mirror commonly cited in older SUMO/OWL literature.
const SUMO_OWL_NAMESPACE: &str = "http://www.ontologyportal.org/SUMO.owl#";

/// The Open English WordNet synset-resource namespace — SEE the module doc;
/// re-verified live 2026-07-14 and matches OEWN's own published RDF prefix.
const OEWN_ID_NAMESPACE: &str = "https://en-word.net/id/";

/// SSSOM's `MappingSet.license` — a `NonRelativeURI` per the spec model, not
/// free text. The exact GPL landing page BOTH of SUMO's own copyright
/// notices cite verbatim (`crates/domains/data/sumo/sumo-LICENSE.txt` quotes
/// both: the repo root README and each `WordNetMappings30-*.txt` header),
/// unversioned because the source itself never specifies a GPL version (the
/// license file's own documented ambiguity, not introduced here). The fuller
/// provenance (pinned commit `152b9abc440477073b5e4e573983b730a619da7b`,
/// the NC-vs-GPL precedence note, the extraction method) lives in the
/// already-cited `sumo-LICENSE.txt` — SSSOM's `license` field is a pointer,
/// not a place to embed a whole license file.
const SUMO_LICENSE_URI: &str = "http://www.gnu.org/copyleft/gpl.html";

/// The set of `ConceptId` VALUES whose EQ rows carry TWO OR MORE DISTINCT
/// SUMO terms — genuinely ambiguous per the source data itself (see the
/// module doc). DERIVED fresh from `mappings` on every call: groups every
/// [`SumoRelationKind::Equivalence`] row by concept, collects each concept's
/// DISTINCT term set, and returns the concepts whose set has more than one
/// member. Never a hardcoded literal list — the
/// `excludes_all_five_known_ambiguous_concepts` test re-derives this same set
/// from the real loaded data and only THEN checks it against the five values
/// the design investigation found, so a future data regen that changes which
/// concepts are ambiguous is caught, not silently stale.
#[must_use]
pub fn ambiguous_eq_concepts(
    mappings: &crate::cognitive::linguistics::sumo::ontology::Sumo,
) -> BTreeSet<u64> {
    let mut terms_by_concept: BTreeMap<u64, BTreeSet<&str>> = BTreeMap::new();
    for m in &mappings.mappings {
        if m.relation == SumoRelationKind::Equivalence {
            terms_by_concept
                .entry(m.concept.value())
                .or_default()
                .insert(m.term.as_str());
        }
    }
    terms_by_concept
        .into_iter()
        .filter(|(_, terms)| terms.len() > 1)
        .map(|(concept, _)| concept)
        .collect()
}

/// Mint the ONE SSSOM `MappingSet` this codebase produces today: every
/// SUMO→WordNet [`SumoRelationKind::Equivalence`] row, MINUS the ambiguous
/// concepts ([`ambiguous_eq_concepts`]), as `skos:exactMatch`
/// [`SssomMapping`]s. Reads [`super::store::sumo_mappings`] — the SAME
/// committed, loaded data [`super::store::sumo_loaded`] indexes for
/// `shares_sumo_class`/`has_coverage` (unaffected by this module: this reads
/// the flat list, never the indexed store).
#[must_use]
pub fn sumo_eq_sssom_mapping_set() -> SssomMappingSet {
    let sumo = super::store::sumo_mappings();
    let ambiguous = ambiguous_eq_concepts(sumo);

    // One row per unambiguous EQ concept. Dedup by concept: the committed TSV
    // is already row-deduped (`regenerate.rs`'s `rows.dedup()`), so a
    // non-ambiguous concept has exactly one EQ row to begin with — the
    // `BTreeMap` here is a defensive single-pass collapse, not a data-quality
    // workaround.
    let mut by_concept: BTreeMap<u64, (&str, &str)> = BTreeMap::new();
    for m in &sumo.mappings {
        if m.relation != SumoRelationKind::Equivalence {
            continue;
        }
        if ambiguous.contains(&m.concept.value()) {
            continue;
        }
        by_concept
            .entry(m.concept.value())
            .or_insert((m.term.as_str(), m.oewn_synset_id.as_str()));
    }

    let predicate_id = predicate_id();

    let mut mappings: Vec<SssomMapping> = by_concept
        .into_values()
        .map(|(term, oewn_synset_id)| SssomMapping {
            subject_id: format!("{OEWN_ID_NAMESPACE}{oewn_synset_id}"),
            predicate_id: predicate_id.clone(),
            object_id: format!("{SUMO_OWL_NAMESPACE}{term}"),
            mapping_justification: MappingJustification::ManualMappingCuration,
        })
        .collect();
    // Canonical, deterministic order (by subject_id) — both for a stable
    // `mapping_set_id` (see below) and a stable public iteration order.
    mappings.sort_by(|a, b| a.subject_id.cmp(&b.subject_id));

    let mapping_set_id = content_addressed_mapping_set_id(&mappings);

    SssomMappingSet {
        mapping_set_id,
        license: SUMO_LICENSE_URI.to_string(),
        mappings,
    }
}

/// SSSOM's `predicate_id` for these mappings — `skos:exactMatch`, sourced
/// from the ALREADY-REGISTERED [`equivalence_relation_kind`]'s own citation
/// (its `ConceptRef` identifies `RelationsConcept::Equivalence`, whose
/// `Relations` ontology definition text names `SKOS exactMatch` verbatim —
/// see `predicate_id_matches_the_registered_equivalence_relation_kind`), not
/// a disconnected literal. Returns the [`EXACT_MATCH_PREDICATE_ID`] constant;
/// the [`equivalence_relation_kind`] call exists to keep this producer
/// grounded to the registered relation concept (and to fail loudly at the
/// call site if that concept is ever renamed).
fn predicate_id() -> String {
    let _kind = equivalence_relation_kind();
    EXACT_MATCH_PREDICATE_ID.to_string()
}

/// A deterministic `mapping_set_id` derived from the mapping set's own
/// content via [`pr4xis_runtime::address::ContentAddress`] — the existing
/// addressing primitive every other content-addressed artifact in this
/// codebase uses, reused rather than inventing a new URL/URN scheme.
/// `mappings` MUST already be in a canonical (sorted) order — the caller
/// ([`sumo_eq_sssom_mapping_set`]) sorts by `subject_id` before calling this,
/// so the SAME input mapping set always hashes to the SAME id (proven by
/// `mapping_set_id_is_content_addressed_and_deterministic`).
fn content_addressed_mapping_set_id(mappings: &[SssomMapping]) -> String {
    let mut canonical = String::new();
    for m in mappings {
        canonical.push_str(&m.subject_id);
        canonical.push('\t');
        canonical.push_str(&m.predicate_id);
        canonical.push('\t');
        canonical.push_str(&m.object_id);
        canonical.push('\t');
        canonical.push_str(m.mapping_justification.curie());
        canonical.push('\n');
    }
    let address = pr4xis_runtime::address::ContentAddress::of(canonical.as_bytes());
    format!("urn:pr4xis:sssom:sumo-wordnet-eq:{}", address.to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::sumo::store::sumo_mappings;

    /// SKOS's own namespace — the CURIE prefix [`EXACT_MATCH_PREDICATE_ID`]
    /// (`skos:exactMatch`) resolves against in this producer's SSSOM export
    /// ([`SUMO_SSSOM_CURIE_MAP`]). SKOS's own spec
    /// (`http://www.w3.org/2004/02/skos/core#`) publishes this exact
    /// namespace. Test-scoped: only the `#[ignore]`d regen test and the
    /// staleness-guard test call [`SssomMappingSet::to_sssom_tsv`].
    const SKOS_CURIE_IRI: &str = "http://www.w3.org/2004/02/skos/core#";

    /// SEMAPV's own namespace — the CURIE prefix
    /// [`MappingJustification::ManualMappingCuration`]'s `semapv:` resolves
    /// against in this producer's SSSOM export ([`SUMO_SSSOM_CURIE_MAP`]).
    /// Independently re-verified live 2026-07-14 against the SEMAPV
    /// ontology's own primary namespace declaration
    /// (`mapping-commons/semantic-mapping-vocabulary`'s `semapv.owl`,
    /// `xmlns:semapv="https://w3id.org/semapv/vocab/"` — every term's
    /// `rdf:about`, e.g. `ManualMappingCuration`, resolves under this exact
    /// base).
    const SEMAPV_CURIE_IRI: &str = "https://w3id.org/semapv/vocab/";

    /// The `curie_map` [`sumo_eq_sssom_mapping_set`]'s SSSOM TSV export
    /// needs — exactly the two prefixes its mappings actually use: `skos:`
    /// (every `predicate_id`) and `semapv:` (every `mapping_justification`).
    /// Passed to [`SssomMappingSet::to_sssom_tsv`], which re-sorts it for
    /// deterministic output regardless of this array's own order.
    const SUMO_SSSOM_CURIE_MAP: [(&str, &str); 2] =
        [("skos", SKOS_CURIE_IRI), ("semapv", SEMAPV_CURIE_IRI)];

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn predicate_id_matches_the_registered_equivalence_relation_kind() {
        use crate::formal::relations::ontology::RelationsConcept;
        use pr4xis::category::Concept;

        // `equivalence_relation_kind()` identifies `RelationsConcept::Equivalence`
        // — cross-check the literal predicate this module emits against THAT
        // concept's own registered definition text, never a disconnected hardcode.
        let kind = equivalence_relation_kind();
        assert_eq!(kind.name, "Equivalence");
        let def = RelationsConcept::Equivalence
            .lexical()
            .expect("Equivalence carries a Relations lexical entry")
            .definition
            .to_string();
        assert!(
            def.contains("exactMatch"),
            "Relations' own Equivalence definition must name SKOS exactMatch; got {def:?}"
        );
        assert_eq!(predicate_id(), "skos:exactMatch");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn excludes_all_five_known_ambiguous_concepts() {
        let sumo = sumo_mappings();
        let ambiguous = ambiguous_eq_concepts(sumo);
        // The design investigation's five candidates — verify they are STILL
        // exactly the ambiguous set the real, current, regenerated data
        // produces (a re-derivation check, not a trust of the literal).
        let expected: BTreeSet<u64> = [95581, 78334, 55582, 36192, 23793].into_iter().collect();
        assert_eq!(
            ambiguous, expected,
            "ambiguous EQ concept set drifted from the design investigation's five candidates \
             — re-derive and update this test's expectation from the real data"
        );

        // Build the REAL synset ids the ambiguous concepts would carry,
        // straight from the source data — never a synthetic numeric guess.
        // `ConceptId` is NOT a WordNet offset (see
        // project_wordnet_source_resolution's own "ConceptId value is
        // load-path-stable", not offset-preserving), so zero-padding a
        // concept id and checking it as a synset-id prefix would never
        // match any real subject_id — a vacuous check that always passes
        // regardless of whether the exclusion actually works. This instead
        // derives the true synset ids ambiguous concepts DO carry and
        // checks those, so a broken filter is genuinely caught.
        let ambiguous_synset_ids: BTreeSet<&str> = sumo
            .mappings
            .iter()
            .filter(|m| {
                m.relation == SumoRelationKind::Equivalence
                    && ambiguous.contains(&m.concept.value())
            })
            .map(|m| m.oewn_synset_id.as_str())
            .collect();
        assert!(
            !ambiguous_synset_ids.is_empty(),
            "sanity: the ambiguous concepts must have real synset ids to check against"
        );

        let set = sumo_eq_sssom_mapping_set();
        for m in &set.mappings {
            let synset_id = m.subject_id.trim_start_matches(OEWN_ID_NAMESPACE);
            assert!(
                !ambiguous_synset_ids.contains(synset_id),
                "an ambiguous concept's synset id leaked into the mapping set: {m:?}"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mapping_set_has_5330_records() {
        let set = sumo_eq_sssom_mapping_set();
        assert_eq!(
            set.mappings.len(),
            5330,
            "5,335 distinct EQ concepts - 5 ambiguous = 5,330 clean 1:1 mappings expected \
             (re-derive this number from the real regenerated data if it drifts)"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_subject_id_is_unique() {
        let set = sumo_eq_sssom_mapping_set();
        let unique: BTreeSet<&str> = set.mappings.iter().map(|m| m.subject_id.as_str()).collect();
        assert_eq!(unique.len(), set.mappings.len());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn subject_id_matches_the_live_en_word_net_uri_pattern() {
        let set = sumo_eq_sssom_mapping_set();
        for m in set.mappings.iter().take(50) {
            assert!(
                m.subject_id.starts_with("https://en-word.net/id/oewn-"),
                "subject_id {} does not match the OEWN URI pattern",
                m.subject_id
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn object_id_matches_the_sumo_owl_uri_pattern() {
        let set = sumo_eq_sssom_mapping_set();
        for m in set.mappings.iter().take(50) {
            assert!(
                m.object_id
                    .starts_with("http://www.ontologyportal.org/SUMO.owl#"),
                "object_id {} does not match the SUMO OWL namespace pattern",
                m.object_id
            );
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn mapping_set_id_is_content_addressed_and_deterministic() {
        let a = sumo_eq_sssom_mapping_set();
        let b = sumo_eq_sssom_mapping_set();
        assert_eq!(a.mapping_set_id, b.mapping_set_id);
        assert!(
            a.mapping_set_id
                .starts_with("urn:pr4xis:sssom:sumo-wordnet-eq:")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn license_is_a_real_uri_matching_sumo_own_cited_gpl_page() {
        let set = sumo_eq_sssom_mapping_set();
        assert_eq!(
            set.license, "http://www.gnu.org/copyleft/gpl.html",
            "the exact GPL URL both of SUMO's own copyright notices cite verbatim"
        );
        assert!(
            set.license.starts_with("http"),
            "SSSOM's MappingSet.license is a NonRelativeURI, never free text"
        );
    }

    /// Path to the committed SSSOM TSV export:
    /// `crates/domains/data/sumo/sumo-wordnet.sssom.tsv`.
    fn committed_sssom_tsv_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/sumo/sumo-wordnet.sssom.tsv")
    }

    /// Regenerate the committed SSSOM TSV export from the loaded SUMO EQ
    /// mapping set ([`sumo_eq_sssom_mapping_set`]) via
    /// [`SssomMappingSet::to_sssom_tsv`]. Run manually after any change that
    /// affects that mapping set's output (a SUMO data regen, a change to
    /// [`ambiguous_eq_concepts`], …):
    /// `cargo test -p pr4xis-domains --features prx -- --ignored regenerate_sumo_wordnet_sssom_tsv`.
    /// The staleness guard below
    /// (`committed_sumo_wordnet_sssom_tsv_matches_source`) fails until this is
    /// re-run and the new file committed.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore]
    fn regenerate_sumo_wordnet_sssom_tsv() {
        let set = sumo_eq_sssom_mapping_set();
        let text = set.to_sssom_tsv(&SUMO_SSSOM_CURIE_MAP);
        let out = committed_sssom_tsv_path();
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).expect("create data/sumo/");
        }
        std::fs::write(&out, text.as_bytes())
            .unwrap_or_else(|e| panic!("write {}: {e}", out.display()));
        eprintln!(
            "wrote {} ({} bytes, {} mappings)",
            out.display(),
            text.len(),
            set.mappings.len()
        );
    }

    /// STALENESS GUARD (normal suite): the committed
    /// `data/sumo/sumo-wordnet.sssom.tsv` must be a FRESH export of the
    /// loaded SUMO EQ mapping set — re-derive it every run and assert
    /// byte-identity with the committed file. HARD-FAILS (no skip) if a
    /// future SUMO data regen (or an `ambiguous_eq_concepts` change) silently
    /// changes the mapping set without regenerating this export — this is a
    /// DERIVED artifact computed from already-pinned SUMO data, not an
    /// independently-fetched external source, so a staleness guard is the
    /// correct discipline (not the registry.toml/praxis.lock fetch-and-pin
    /// cascade).
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn committed_sumo_wordnet_sssom_tsv_matches_source() {
        let set = sumo_eq_sssom_mapping_set();
        let fresh = set.to_sssom_tsv(&SUMO_SSSOM_CURIE_MAP);
        let committed = std::fs::read_to_string(committed_sssom_tsv_path())
            .expect("read committed data/sumo/sumo-wordnet.sssom.tsv");
        assert_eq!(
            fresh, committed,
            "committed data/sumo/sumo-wordnet.sssom.tsv is STALE — regenerate with \
             `cargo test -p pr4xis-domains --features prx -- --ignored regenerate_sumo_wordnet_sssom_tsv`"
        );
    }
}
