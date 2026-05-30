//! Statute → `OntologyBuilder` codegen.
//!
//! Mirrors the [`super::wordnet`] codegen path but for legal statutes.
//! Input: a structural JSON file (praxis's statute schema: `terms[]` +
//! `relations[]`) that names the statute's concepts and the relations
//! between them. The verbatim statutory text lives in a sibling `.txt`
//! file (fetched via [`data_provisioning`][crate-doc] and hash-verified);
//! this parser does not read it directly — it only consumes the
//! structured terms.
//!
//! Output: an [`OntologyBuilder`] populated with entities + relations,
//! ready for [`generate_rust`][super::generate_rust] to emit a static
//! Rust module per statute.
//!
//! [crate-doc]: ../../../pr4xis_domains/applied/data_provisioning/index.html
//!
//! # Relation-type mapping
//!
//! Statutes carry richer relation semantics than the five canonical
//! kinds [`OntologyBuilder`] currently models. The 13 statute relation
//! types in the structural JSON map as follows:
//!
//! | JSON `relation` | OntologyBuilder kind | Rationale |
//! |---|---|---|
//! | `SubtypeOf` | `taxonomy` | A is-a B |
//! | `Requires` | `mereology` | Requiring term has the required term as a component |
//! | `Composes` | `mereology` | A composes-into B means B has-a A |
//! | `SafeHarborFor` | `mereology` | Safe-harbor scenario contains the protected behavior |
//! | `Contradicts` | `opposition` | Direct antinomy |
//! | `Negates` | `opposition` | Burden-shift opposition |
//! | `Rebuts` | `opposition` | Defeats a burden |
//! | `AffirmativeDefenseTo` | `opposition` | Defeats a claim even if elements proven |
//! | `Implies` | `causation` | A causes B's legal consequence |
//! | `Triggers` | `causation` | A triggers an obligation |
//! | `Precedes` | `causation` | Temporal precedence is causal antecedent |
//! | `ExhaustionRequiredFor` | `causation` | Exhaustion temporally / causally enables |
//! | `AlternativeTo` | `equivalence` | Alternatives are interchangeable choices |
//!
//! This mapping is **lossy** — the JSON fields `max_days`, `consequence`,
//! `obligation`, `into`, `burden` (which qualify some relation variants)
//! are dropped at this layer. They remain queryable on the source JSON
//! and can be lifted into the generated ontology in a future enhancement
//! once [`OntologyBuilder`] grows arbitrary-kind support. For the
//! current first-cut codegen, the structural shape of the legal claim
//! survives the mapping; the parametric details do not.
//!
//! # Input schema (abbreviated)
//!
//! ```text
//! {
//!   "name": "sox_1514a",
//!   "description": "18 U.S.C. § 1514A — …",
//!   "terms": [
//!     { "id": "sox_1514a:a",
//!       "name": "Covered Employer",
//!       "definition": "A company with registered securities…",
//!       "subsection": "(a)",
//!       … (other fields ignored at this layer) }
//!   ],
//!   "relations": [
//!     { "from": "sox_1514a:a_v3",
//!       "to":   "sox_1514a:a",
//!       "relation": { "Composes": { "into": "claim" } } }
//!   ]
//! }
//! ```

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::path::Path;

use serde::Deserialize;

use super::builder::{EntityDef, OntologyBuilder};

/// Errors that can arise while parsing a structural statute JSON file.
#[derive(Debug)]
pub enum ParseError {
    /// Failed to read the file.
    Read(String, std::io::Error),
    /// JSON-parse failure.
    Json(serde_json::Error),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Read(path, e) => write!(f, "read {path}: {e}"),
            Self::Json(e) => write!(f, "parse JSON: {e}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a structural-statute JSON file into an [`OntologyBuilder`].
/// Each `term` becomes an `EntityDef`; each `relation` becomes a kinded
/// edge per the mapping in the module-level doc table.
pub fn parse_statute_json(path: &Path) -> Result<OntologyBuilder, ParseError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| ParseError::Read(path.display().to_string(), e))?;
    let doc: RawStatuteDoc = serde_json::from_str(&raw).map_err(ParseError::Json)?;
    Ok(build_from_doc(&doc))
}

/// Build an [`OntologyBuilder`] from a parsed `RawStatuteDoc`. Public
/// so callers that obtain the doc through some other path (e.g.,
/// pr4xis-domains' build.rs reading praxis.lock TOML and converting in
/// memory) can drive the codegen without round-tripping through a JSON
/// file on disk.
pub fn build_from_doc(doc: &RawStatuteDoc) -> OntologyBuilder {
    let mut b = OntologyBuilder::new();

    for term in &doc.terms {
        let mut ent = EntityDef::new(&term.id, &term.name);
        // Distinguish statute entities from lexical entities (WordNet
        // uses "n"/"v"/"adj"/"adv"). "statute_term" is a synthetic POS
        // tag that downstream consumers can pattern-match.
        ent = ent.pos("statute_term");
        ent = ent.definition(&term.definition);
        b.add_entity(ent);

        // Each lemma becomes a word-index entry pointing at this term.
        // This is the seed for the statute↔English adjunction:
        // downstream codegen + functor synthesis can look up matching
        // WordNet entities by lemma. (Adjunction generation itself is
        // a separate codegen pass; this layer just records the raw
        // lemmas.)
        for lemma in &term.lemmas {
            b.add_word_index(lemma, &term.id);
        }
    }

    for rel in &doc.relations {
        let from = rel.from.as_str();
        let to = rel.to.as_str();
        match &rel.relation {
            RawRel::SubtypeOf => {
                b.add_taxonomy(from, to);
            }
            RawRel::Requires | RawRel::Composes { .. } | RawRel::SafeHarborFor => {
                b.add_mereology(from, to);
            }
            RawRel::Contradicts
            | RawRel::Negates
            | RawRel::Rebuts { .. }
            | RawRel::AffirmativeDefenseTo => {
                b.add_opposition(from, to);
            }
            RawRel::Implies { .. }
            | RawRel::Triggers { .. }
            | RawRel::Precedes { .. }
            | RawRel::ExhaustionRequiredFor => {
                b.add_causation(from, to);
            }
            RawRel::AlternativeTo => {
                b.add_equivalence(from, to);
            }
        }
    }

    b
}

// ---------------------------------------------------------------------------
// Statute structural-JSON schema (serde-deserialized).
//
// Fields not consumed at this layer (e.g. `valence`, `obligations`,
// `deadlines`, `rights`, `remedies`, `burdens`, `exceptions`,
// `required_evidence`) are accepted-and-ignored. They're not lost — the
// source JSON is preserved as a data file in
// `crates/domains/data/statutes/` and a future codegen pass can lift
// them into the generated ontology (e.g. as `Quality` impls) when
// downstream consumers need them.
// ---------------------------------------------------------------------------

/// Parsed statute structural document. Public so callers driving the
/// codegen from in-memory data (e.g., pr4xis-domains' build.rs reading
/// TOML structural blocks from praxis.lock) can construct an instance
/// and feed it to [`build_from_doc`].
#[derive(Debug, Default, Deserialize)]
pub struct RawStatuteDoc {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub terms: Vec<RawTerm>,
    #[serde(default)]
    pub relations: Vec<RawRelation>,
}

/// One statutory term. Surface a public constructor for in-memory use.
#[derive(Debug, Deserialize)]
pub struct RawTerm {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub definition: String,
    /// Surface-form lemmas the term uses. Optional in the schema;
    /// populated for the cases where the source carries them. Seed for
    /// the statute↔English adjunction (downstream codegen pass).
    #[serde(default)]
    pub lemmas: Vec<String>,
}

/// One relation between two statutory terms.
#[derive(Debug, Deserialize)]
pub struct RawRelation {
    pub from: String,
    pub to: String,
    pub relation: RawRel,
}

/// The 13 statute relation types. The on-disk format serialises
/// variants as struct-tagged PascalCase (`{"Composes": {"into":
/// "claim"}}`), which serde's default external tagging produces
/// naturally for enums.
#[derive(Debug, Deserialize)]
pub enum RawRel {
    Requires,
    SubtypeOf,
    Contradicts,
    Negates,
    AlternativeTo,
    AffirmativeDefenseTo,
    SafeHarborFor,
    ExhaustionRequiredFor,
    Precedes {
        #[allow(dead_code)]
        max_days: Option<u32>,
    },
    Implies {
        #[allow(dead_code)]
        consequence: Option<String>,
    },
    Composes {
        #[allow(dead_code)]
        into: Option<String>,
    },
    Triggers {
        #[allow(dead_code)]
        obligation: Option<String>,
    },
    Rebuts {
        #[allow(dead_code)]
        burden: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"{
        "name": "test_statute",
        "description": "Synthetic test statute.",
        "terms": [
            {
                "id": "test:a",
                "name": "Protected Activity",
                "definition": "Activity protected from retaliation.",
                "lemmas": ["report", "disclose"]
            },
            {
                "id": "test:a_v2",
                "name": "Adverse Action",
                "definition": "Discharge, demotion, or harassment.",
                "lemmas": ["discharge", "demote"]
            },
            {
                "id": "test:a_v3",
                "name": "Causation",
                "definition": "Because-of nexus.",
                "lemmas": []
            },
            {
                "id": "test:claim",
                "name": "Prima Facie Claim",
                "definition": "The composed claim.",
                "lemmas": []
            }
        ],
        "relations": [
            { "from": "test:a", "to": "test:claim",
              "relation": { "Composes": { "into": "claim" } } },
            { "from": "test:a_v2", "to": "test:claim",
              "relation": { "Composes": { "into": "claim" } } },
            { "from": "test:a_v3", "to": "test:claim",
              "relation": { "Composes": { "into": "claim" } } },
            { "from": "test:a", "to": "test:a_v2",
              "relation": "Contradicts" }
        ]
    }"#;

    #[test]
    fn parses_terms_into_entities() {
        let doc: RawStatuteDoc = serde_json::from_str(SAMPLE_JSON).unwrap();
        let b = build_from_doc(&doc);
        assert_eq!(b.entities.len(), 4);
        assert!(b.entities.iter().any(|e| e.id == "test:claim"));
        assert!(
            b.entities
                .iter()
                .find(|e| e.id == "test:a")
                .map(|e| e.lemmas.is_empty())
                .unwrap_or(true)
        );
        // EntityDef stores lemmas separately on the entity; the
        // builder also seeds word_index.
        assert!(b.word_index.iter().any(|(w, _)| w == "report"));
        assert!(b.word_index.iter().any(|(w, _)| w == "discharge"));
    }

    #[test]
    fn composes_relation_maps_to_mereology() {
        let doc: RawStatuteDoc = serde_json::from_str(SAMPLE_JSON).unwrap();
        let b = build_from_doc(&doc);
        // Three Composes → three mereology edges.
        assert_eq!(b.mereology.len(), 3);
        assert!(
            b.mereology
                .iter()
                .any(|(a, b)| a == "test:a" && b == "test:claim")
        );
    }

    #[test]
    fn contradicts_relation_maps_to_opposition() {
        let doc: RawStatuteDoc = serde_json::from_str(SAMPLE_JSON).unwrap();
        let b = build_from_doc(&doc);
        assert_eq!(b.opposition.len(), 1);
        assert!(
            b.opposition
                .iter()
                .any(|(a, b)| a == "test:a" && b == "test:a_v2")
        );
    }

    #[test]
    fn relation_count_aggregates() {
        let doc: RawStatuteDoc = serde_json::from_str(SAMPLE_JSON).unwrap();
        let b = build_from_doc(&doc);
        // 3 mereology + 1 opposition.
        assert_eq!(b.relation_count(), 4);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        // Real-world statute JSONs may carry additional fields like
        // valence, obligations, deadlines, etc. We accept-and-ignore
        // them at this layer; a later codegen pass will lift them.
        let json = r#"{
            "name": "test",
            "description": "x",
            "authority": {"Constitution": {"provision": "see source"}},
            "terms": [{
                "id": "x:1",
                "name": "X",
                "definition": "x def",
                "valence": "Supportive",
                "subsection": "(a)",
                "required_evidence": [],
                "obligations": [],
                "deadlines": []
            }],
            "relations": []
        }"#;
        let doc: RawStatuteDoc = serde_json::from_str(json).unwrap();
        let b = build_from_doc(&doc);
        assert_eq!(b.entities.len(), 1);
        assert_eq!(b.entities[0].id, "x:1");
    }
}
