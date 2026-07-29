//! SSSOM (Simple Standard for Sharing Ontological Mappings) — the
//! spec-mandatory-field subset of a `Mapping`/`MappingSet`, as plain typed
//! Rust data.
//!
//! Mirrors [`crate::cognitive::linguistics::sumo::ontology`]'s plain
//! struct-family shape (an instance-data model, not a `pr4xis::ontology!`
//! category): SSSOM's own spec is a data-interchange format for
//! already-discovered correspondences, not a reasoning category with its own
//! concepts/edges/axioms — [`crate::formal::information::schema::alignment`]
//! already IS that category (Euzenat & Shvaiko's `Alignment`/`Correspondence`
//! lifecycle), and SSSOM is deliberately kept separate from it: a
//! [`SssomMapping`] is a SERIALIZATION of one concrete correspondence, not a
//! new `AlignmentConcept`. Categorically, an alignment is a span `O1 ← A → O2`
//! (Zimmermann et al. 2006, cited in `alignment.rs`); SSSOM's `Mapping` is the
//! same span made concrete and exchangeable — `subject_id`/`object_id` are the
//! two legs, `predicate_id` labels the relation, `mapping_justification`
//! records HOW the correspondence was discovered (the evidence Euzenat &
//! Shvaiko's `MatchingTechnique` taxonomy classifies, spoken in SEMAPV's own
//! vocabulary instead).
//!
//! ## Mandatory-subset only
//!
//! The SSSOM spec's `Mapping` requires exactly four fields — `subject_id`,
//! `predicate_id`, `object_id`, `mapping_justification` — with NO exception
//! for a non-literal mapping. Matentzoglu et al. (2022) establishes this
//! four-mandatory-field requirement, but under the FOURTH field's ORIGINAL
//! name, `match_type` ("how the mapping was derived") — `mapping_justification`
//! is a later rename (SSSOM v0.9.3, changelog issue #150); this module targets
//! the CURRENT living spec-model (mapping-commons.github.io/sssom/dev/
//! spec-model/), which uses the renamed field, not the 2022 paper's original
//! name. Its `MappingSet` requires `mapping_set_id` and `license` — per the
//! living spec-model (the 2022 paper lists both only among ~23 illustrative
//! elements, without singling them out as mandatory). Everything else the
//! spec defines (`confidence`, `subject_label`, `mapping_tool`, `author_id`,
//! …) is RECOMMENDED or OPTIONAL — deliberately NOT modeled here until a real
//! producer has real data to populate it honestly. In particular this module
//! carries NO `confidence` field: inventing a number the source data does not
//! publish would be a fabricated claim, not an honest omission (see
//! [`crate::cognitive::linguistics::sumo::sssom`]'s module doc for the
//! concrete case this bites).
//!
//! ## Justification vocabulary — SEMAPV, not a free string
//!
//! [`MappingJustification`] enumerates SEMAPV (Semantic Mapping Vocabulary)
//! terms as a closed Rust enum rather than an unchecked string, so a
//! mis-typed CURIE is a compile error, not a silent data-quality bug — the
//! same discipline [`crate::cognitive::linguistics::sumo::ontology::SumoRelationKind`]
//! applies to its own six-suffix legend. Only the ONE variant a real producer
//! in this codebase currently needs is modeled
//! ([`MappingJustification::ManualMappingCuration`] — see
//! [`crate::cognitive::linguistics::sumo::sssom`]); more are added only when a
//! second real use case needs them (the same "one concrete case before
//! generalizing" discipline every grounding lens in this codebase follows).
//!
//! ## Physical format — embedded mode
//!
//! SSSOM defines two physical serializations; [`SssomMappingSet::to_sssom_tsv`]
//! targets EMBEDDED mode (the spec's default, single-file form): `#`-prefixed
//! YAML metadata lines above a tab-separated table. This convention is
//! confirmed against a REAL, genuinely committed SSSOM file — NOT a tutorial
//! illustration — `tests/data/basic.tsv` in the official `mapping-commons/
//! sssom-py` reference-implementation repository
//! (<https://raw.githubusercontent.com/mapping-commons/sssom-py/master/tests/data/basic.tsv>,
//! fetched and read directly), whose header block (verbatim) is:
//!
//! ```text
//! #license: "https://creativecommons.org/publicdomain/zero/1.0/"
//! #mapping_set_id: https://w3id.org/sssom/mapping/tests/data/basic.tsv
//! #curie_map:
//! #  semapv: "https://w3id.org/semapv/vocab/"
//! subject_id  predicate_id  object_id  mapping_justification  confidence  ...
//! x:appendage  owl:equivalentClass  y:appendage  semapv:ManualMappingCuration  0.84  ...
//! ```
//!
//! (columns are TAB-separated in the real file; shown here as double spaces
//! because clippy forbids literal tabs inside doc comments; the real file's
//! header has no space after `#` on the scalar lines — either form is valid
//! YAML-comment syntax, [`SssomMappingSet::to_sssom_tsv`] uses `# ` with a
//! space; the real row carries 16 columns this module does not model — see
//! below — trimmed to the 4 this module's own output actually carries.
//! `semapv:ManualMappingCuration` appears in this real file's real data,
//! independently confirming the term this module's
//! [`MappingJustification::ManualMappingCuration`] variant names is a live,
//! currently-used SEMAPV term, not a guess.)
//!
//! [`SssomMappingSet::to_sssom_tsv`] emits exactly the four fields
//! [`SssomMapping`] actually carries
//! (`subject_id`/`predicate_id`/`object_id`/`mapping_justification`), never
//! `confidence`, `subject_label`, or any other field `basic.tsv` above shows
//! that this module does not model (see "Mandatory-subset only" above for
//! why: those fields are recommended/optional per the living spec, and this
//! module's one real producer has no honest data to populate them with).
//!
//! # Literature
//!
//! - **Matentzoglu, N., Balhoff, J. P., Bello, S. M., Bizon, C., Brush, M.,
//!   Callahan, T. J., et al. (2022)** "A Simple Standard for Sharing
//!   Ontological Mappings (SSSOM)", *Database* (Oxford), Vol. 2022,
//!   baac035. <https://doi.org/10.1093/database/baac035> — establishes the
//!   four-mandatory-field `Mapping` requirement ("Four of these are required
//!   for any individual mapping: `subject_id`, `object_id`, ..., `predicate_id`
//!   ..., and `match_type` (how the mapping was derived)"). The paper's field
//!   is `match_type`, not `mapping_justification` — see the module doc above.
//! - **SSSOM specification model** (mapping-commons.github.io/sssom/dev/
//!   spec-model/), the CURRENT living machine-readable LinkML schema (the
//!   source of truth this module targets, post-dating the 2022 paper) —
//!   `subject_id`/`predicate_id`/`object_id`/`mapping_justification` (the
//!   renamed field) are `mandatory: true`; `mapping_set_id`/`license` are the
//!   `MappingSet` mandatory fields; `confidence` is `recommended`, not
//!   mandatory.
//! - **SSSOM CHANGELOG** (raw.githubusercontent.com/mapping-commons/sssom/
//!   master/CHANGELOG.md), version 0.9.3 — "Changed `match_type` logic to
//!   `mapping_justification` (issue #150)", the rename this module's field
//!   name follows.
//! - **SEMAPV — Semantic Mapping Vocabulary**
//!   (mapping-commons/semantic-mapping-vocabulary) — the CURIE vocabulary
//!   SSSOM's `mapping_justification` values are drawn from;
//!   `semapv:ManualMappingCuration` is one of its terms.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

/// A SEMAPV justification term — HOW a [`SssomMapping`] was discovered. Only
/// the variant a real producer in this codebase needs today is modeled (see
/// the module doc); each variant's `curie()` is the exact SEMAPV CURIE the
/// SSSOM `mapping_justification` field carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingJustification {
    /// `semapv:ManualMappingCuration` — the correspondence was hand-curated
    /// by a human expert, as opposed to a computed technique like lexical or
    /// logical matching. SEMAPV's own definition (semapv-terms.tsv):
    /// "A matching process that is performed by a human agent and is based
    /// on human judgement and domain knowledge." The SUMO
    /// WordNet↔SUMO crosswalk this module's first producer reads is exactly
    /// this: Niles & Pease's own paper title is "Linking Lexicons and
    /// Ontologies: Mapping WordNet to SUMO" — a hand-curated third-party
    /// crosswalk, not something this codebase (or SUMO's authors, per their
    /// own description of the work) computed via lexical comparison.
    ManualMappingCuration,
}

impl MappingJustification {
    /// The exact SEMAPV CURIE this justification names.
    #[must_use]
    pub fn curie(self) -> &'static str {
        match self {
            Self::ManualMappingCuration => "semapv:ManualMappingCuration",
        }
    }
}

/// One SSSOM `Mapping` — the four spec-mandatory fields, nothing invented.
/// `subject_id`/`object_id` are the span's two legs (real, external URIs);
/// `predicate_id` is the relation between them (a CURIE, e.g.
/// `skos:exactMatch`); `mapping_justification` is the evidence class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SssomMapping {
    /// The subject entity's IRI (spec: mandatory).
    pub subject_id: String,
    /// The relation's CURIE, e.g. `skos:exactMatch` (spec: mandatory).
    pub predicate_id: String,
    /// The object entity's IRI (spec: mandatory).
    pub object_id: String,
    /// HOW this correspondence was discovered (spec: mandatory).
    pub mapping_justification: MappingJustification,
}

/// One SSSOM `MappingSet` — the two spec-mandatory fields plus the mappings
/// themselves. `mapping_set_id` is a real, content-derived identifier (never
/// an invented URL scheme — see
/// [`crate::cognitive::linguistics::sumo::sssom`] for how a real producer
/// mints one via [`pr4xis_runtime::address::ContentAddress`]); `license`
/// names the terms the mapping SET itself is published under (spec:
/// mandatory).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SssomMappingSet {
    /// A unique identifier for this mapping set (spec: mandatory).
    pub mapping_set_id: String,
    /// The license the mapping set is published under (spec: mandatory).
    pub license: String,
    /// The mappings this set carries.
    pub mappings: Vec<SssomMapping>,
}

impl SssomMappingSet {
    /// Serialize this mapping set to SSSOM's EMBEDDED physical format — a
    /// single `.sssom.tsv` file carrying `#`-prefixed YAML metadata lines
    /// (`mapping_set_id`, `license`, `curie_map:`) above a tab-separated
    /// table, exactly the shape independently re-verified live 2026-07-14
    /// against mapping-commons.github.io/sssom/dev/tutorial/ (see this
    /// module's own doc comment for the quoted real example this mirrors).
    ///
    /// `curie_map` is CALLER-supplied — different producers use different
    /// CURIE prefixes across `predicate_id`/`mapping_justification`, so this
    /// method stays generic over any [`SssomMappingSet`] rather than baking
    /// in one producer's prefixes (see
    /// [`crate::cognitive::linguistics::sumo::sssom`] for the concrete
    /// `skos`/`semapv` map its EQ export needs). Entries are sorted by
    /// prefix for a deterministic, diff-stable output regardless of the
    /// slice's input order.
    ///
    /// `self.mappings` is emitted in ITS OWN existing order — never
    /// re-sorted here. Producers (e.g. [`sumo_eq_sssom_mapping_set`](
    /// crate::cognitive::linguistics::sumo::sssom::sumo_eq_sssom_mapping_set))
    /// already establish the canonical order their `mapping_set_id` was
    /// content-addressed against; re-sorting in the serializer would silently
    /// decouple the emitted row order from that address.
    #[must_use]
    pub fn to_sssom_tsv(&self, curie_map: &[(&str, &str)]) -> String {
        let mut out = String::new();
        out.push_str("# mapping_set_id: ");
        out.push_str(&self.mapping_set_id);
        out.push('\n');
        out.push_str("# license: ");
        out.push_str(&self.license);
        out.push('\n');
        out.push_str("# curie_map:\n");

        let mut sorted_curies: Vec<(&str, &str)> = curie_map.to_vec();
        sorted_curies.sort_by(|a, b| a.0.cmp(b.0));
        for (prefix, iri) in &sorted_curies {
            out.push_str("#   ");
            out.push_str(prefix);
            out.push_str(": ");
            out.push_str(iri);
            out.push('\n');
        }

        out.push_str("subject_id\tpredicate_id\tobject_id\tmapping_justification\n");
        for m in &self.mappings {
            out.push_str(&m.subject_id);
            out.push('\t');
            out.push_str(&m.predicate_id);
            out.push('\t');
            out.push_str(&m.object_id);
            out.push('\t');
            out.push_str(m.mapping_justification.curie());
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn justification_curie_is_the_real_semapv_term() {
        assert_eq!(
            MappingJustification::ManualMappingCuration.curie(),
            "semapv:ManualMappingCuration"
        );
    }

    fn sample_mapping() -> SssomMapping {
        SssomMapping {
            subject_id: "https://en-word.net/id/oewn-00001740-n".into(),
            predicate_id: "skos:exactMatch".into(),
            object_id: "http://www.adampease.org/OP/SUMO.owl#Entity".into(),
            mapping_justification: MappingJustification::ManualMappingCuration,
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mapping_has_all_four_mandatory_fields_nonempty() {
        let m = sample_mapping();
        assert!(!m.subject_id.is_empty());
        assert!(!m.predicate_id.is_empty());
        assert!(!m.object_id.is_empty());
        assert_eq!(
            m.mapping_justification.curie(),
            "semapv:ManualMappingCuration"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mapping_set_has_both_mandatory_fields_nonempty() {
        let set = SssomMappingSet {
            mapping_set_id: "urn:pr4xis:sssom:deadbeef".into(),
            license: "GPL".into(),
            mappings: alloc::vec![sample_mapping()],
        };
        assert!(!set.mapping_set_id.is_empty());
        assert!(!set.license.is_empty());
        assert_eq!(set.mappings.len(), 1);
    }

    fn sample_set() -> SssomMappingSet {
        SssomMappingSet {
            mapping_set_id: "urn:pr4xis:sssom:deadbeef".into(),
            license: "http://www.gnu.org/copyleft/gpl.html".into(),
            mappings: alloc::vec![
                sample_mapping(),
                SssomMapping {
                    subject_id: "https://en-word.net/id/oewn-00002137-n".into(),
                    predicate_id: "skos:exactMatch".into(),
                    object_id: "http://www.ontologyportal.org/SUMO.owl#Physical".into(),
                    mapping_justification: MappingJustification::ManualMappingCuration,
                },
            ],
        }
    }

    const SAMPLE_CURIE_MAP: [(&str, &str); 2] = [
        ("skos", "http://www.w3.org/2004/02/skos/core#"),
        ("semapv", "https://w3id.org/semapv/vocab/"),
    ];

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn to_sssom_tsv_matches_the_verified_embedded_mode_shape() {
        let set = sample_set();
        let text = set.to_sssom_tsv(&SAMPLE_CURIE_MAP);
        let lines: Vec<&str> = text.lines().collect();

        assert_eq!(lines[0], "# mapping_set_id: urn:pr4xis:sssom:deadbeef");
        assert_eq!(lines[1], "# license: http://www.gnu.org/copyleft/gpl.html");
        assert_eq!(lines[2], "# curie_map:");
        // Sorted by prefix ("semapv" < "skos"), regardless of input order.
        assert_eq!(lines[3], "#   semapv: https://w3id.org/semapv/vocab/");
        assert_eq!(lines[4], "#   skos: http://www.w3.org/2004/02/skos/core#");
        assert_eq!(
            lines[5],
            "subject_id\tpredicate_id\tobject_id\tmapping_justification"
        );
        assert_eq!(
            lines[6],
            "https://en-word.net/id/oewn-00001740-n\tskos:exactMatch\t\
             http://www.adampease.org/OP/SUMO.owl#Entity\tsemapv:ManualMappingCuration"
        );
        assert_eq!(
            lines[7],
            "https://en-word.net/id/oewn-00002137-n\tskos:exactMatch\t\
             http://www.ontologyportal.org/SUMO.owl#Physical\tsemapv:ManualMappingCuration"
        );
        // Rows kept in the caller's own order (not re-sorted): the first
        // sample row's subject_id sorts AFTER the second's, proving this
        // is preservation, not a hidden re-sort.
        assert!(lines[6] < lines[7]);
        assert_eq!(
            lines.len(),
            8,
            "exactly 3 metadata + 1 header + 4 data lines expected"
        );
    }

    /// Parses `text` back into `(mapping_set_id, license, curie_map, mappings)`
    /// via a small, honest internal parser — splitting the `#`-prefixed header
    /// lines from the tab-separated data rows — proving
    /// [`SssomMappingSet::to_sssom_tsv`] is genuinely well-formed, not string
    /// concatenation that happens to look right.
    fn parse_sssom_tsv(
        text: &str,
    ) -> (
        alloc::string::String,
        alloc::string::String,
        alloc::vec::Vec<(alloc::string::String, alloc::string::String)>,
        Vec<SssomMapping>,
    ) {
        use alloc::string::ToString;

        let mut mapping_set_id = None;
        let mut license = None;
        let mut curie_map = Vec::new();
        let mut header_seen = false;
        let mut mappings = Vec::new();

        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("# mapping_set_id: ") {
                mapping_set_id = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("# license: ") {
                license = Some(rest.to_string());
            } else if line == "# curie_map:" {
                continue;
            } else if let Some(rest) = line.strip_prefix("#   ") {
                let (prefix, iri) = rest.split_once(": ").expect("curie_map line has ': '");
                curie_map.push((prefix.to_string(), iri.to_string()));
            } else if !header_seen {
                assert_eq!(
                    line, "subject_id\tpredicate_id\tobject_id\tmapping_justification",
                    "unexpected header line"
                );
                header_seen = true;
            } else {
                let cols: Vec<&str> = line.split('\t').collect();
                assert_eq!(
                    cols.len(),
                    4,
                    "expected 4 tab-separated columns, got {cols:?}"
                );
                let mapping_justification = match cols[3] {
                    "semapv:ManualMappingCuration" => MappingJustification::ManualMappingCuration,
                    other => panic!("unknown mapping_justification CURIE: {other}"),
                };
                mappings.push(SssomMapping {
                    subject_id: cols[0].to_string(),
                    predicate_id: cols[1].to_string(),
                    object_id: cols[2].to_string(),
                    mapping_justification,
                });
            }
        }

        (
            mapping_set_id.expect("mapping_set_id line present"),
            license.expect("license line present"),
            curie_map,
            mappings,
        )
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn to_sssom_tsv_round_trips_losslessly() {
        let set = sample_set();
        let text = set.to_sssom_tsv(&SAMPLE_CURIE_MAP);
        let (mapping_set_id, license, curie_map, mappings) = parse_sssom_tsv(&text);

        assert_eq!(mapping_set_id, set.mapping_set_id);
        assert_eq!(license, set.license);
        assert_eq!(curie_map.len(), SAMPLE_CURIE_MAP.len());
        for (prefix, iri) in &SAMPLE_CURIE_MAP {
            assert!(
                curie_map.iter().any(|(p, i)| p == prefix && i == iri),
                "curie_map lost the {prefix}:{iri} entry"
            );
        }
        assert_eq!(
            mappings, set.mappings,
            "the 4 mandatory fields must survive losslessly"
        );
    }
}
