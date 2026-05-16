//! Statute → English adjunction. Resolves typed `Form` lemmas
//! against the loaded `English` WordNet ontology to produce typed
//! `Sense` references, completing the lexical-level connection from
//! statute terms to English meanings.
//!
//! This is the M5 milestone deliverable from the whistleblowing
//! roadmap: each statute term's name decomposes into content-word
//! lemmas (via [`super::term_extractor::extract_lemmas`]); each
//! lemma resolves to zero or more English WordNet senses via this
//! adjunction. The result is a fully-typed map from
//! `(LegalTerm.id, Form)` → `Vec<Sense>` that downstream gap-
//! detection can use.
//!
//! # Categorical structure
//!
//! ```text
//!   Statute concept       Form (lemma)        English concept
//!  (LegalTerm.id)       (ontolex:Form)      (English::ConceptId via
//!                                          ConceptRef ontology=english_wordnet)
//!         │                   │                    │
//!         │                   │                    │
//!         └── extract_lemmas ─┴── resolve_form_to_senses ──┘
//! ```
//!
//! Mirrors the per-domain adjunction pattern already in
//! `pr4xis-domains::natural::biomedical::adjunctions` — domain
//! concepts flow into the English lexicon via typed `Sense`
//! references, enabling lexical-level reasoning over a domain-
//! specific vocabulary.
//!
//! # Praxis-way typing
//!
//! Inputs are `Form` (ontolex:Form, W3C 2017). Outputs are `Sense`
//! (ontolex:LexicalSense, W3C 2017) carrying a `ConceptRef` with
//! `ontology = "english_wordnet"` and `concept = <synset_id>`. No
//! bare strings escape the API.
//!
//! # Literature
//!
//! - **McCrae et al. (2017)** "The Ontolex-Lemon Model: Development
//!   and Applications" *Proc. eLex 2017* — Form / LexicalEntry /
//!   Sense / Reference architecture.
//! - **Fellbaum, Christiane (ed.) (1998)** *WordNet: An Electronic
//!   Lexical Database*, MIT Press — synset semantics.
//! - **Vossen, Piek (ed.) (1998)** *EuroWordNet: A Multilingual
//!   Database with Lexical Semantic Networks*, Kluwer — cross-
//!   language wordnet structure (the open-corpus version praxis
//!   loads).
//! - **Sartor, Giovanni (2005)** *Legal Reasoning: A Cognitive
//!   Approach to the Law*, Springer (Treatise vol. 5) — Ch. 12
//!   bridge from legal terminology to general-language lexicon.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::cognitive::linguistics::english::ontology::English;
use crate::cognitive::linguistics::lemon::lexicon::{ConceptRef, Form, Sense};

// ─────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────

/// Adjunction output: one `Form` and the typed `Sense`s it resolves
/// to in the loaded English WordNet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LemmaSenseMapping {
    /// The lemma being resolved.
    pub form: Form,
    /// The English senses the lemma maps to (zero if the lemma
    /// isn't in the loaded English ontology).
    pub senses: Vec<Sense>,
}

impl LemmaSenseMapping {
    /// True if the lemma found at least one matching sense.
    pub fn is_resolved(&self) -> bool {
        !self.senses.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Resolution functions
// ─────────────────────────────────────────────────────────────────────

/// Resolve a single `Form` to its English senses.
///
/// Looks up `form.written_rep` in `english.word_index`; for every
/// matching `ConceptId`, builds a typed `Sense` with
/// `reference = ConceptRef { ontology: "english_wordnet", concept:
/// <synset_id> }`. Returns an empty `Vec` if the lemma is unknown.
///
/// `form.lang` is not currently checked — the adjunction assumes
/// English-language `Form`s. Future multi-language wordnets would
/// dispatch on `lang` to the matching wordnet instance.
pub fn resolve_form_to_senses(form: &Form, english: &English) -> Vec<Sense> {
    let concept_ids = english.lookup(&form.written_rep);
    concept_ids
        .iter()
        .filter_map(|id| english.concept(*id))
        .map(|c| Sense {
            reference: ConceptRef {
                ontology: "english_wordnet".to_string(),
                concept: c.original_id.clone(),
            },
        })
        .collect()
}

/// Resolve a list of `Form`s to per-lemma sense mappings.
pub fn resolve_lemmas_to_senses(forms: &[Form], english: &English) -> Vec<LemmaSenseMapping> {
    forms
        .iter()
        .map(|f| LemmaSenseMapping {
            form: f.clone(),
            senses: resolve_form_to_senses(f, english),
        })
        .collect()
}

/// Resolve a statute term name end-to-end:
/// 1. Extract content-word lemmas via [`super::term_extractor::extract_lemmas`].
/// 2. Resolve each lemma against the English ontology.
///
/// Returns one `LemmaSenseMapping` per content lemma.
pub fn resolve_term_name_to_senses(term_name: &str, english: &English) -> Vec<LemmaSenseMapping> {
    let lemmas = super::term_extractor::extract_lemmas(term_name);
    resolve_lemmas_to_senses(&lemmas, english)
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::lmf;

    /// Minimal WordNet covering the lemmas the statute-term tests
    /// need. Standard LMF format; matches what
    /// `cognitive::linguistics::english::tests::sample_english`
    /// uses for shape.
    const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource xmlns:dc="https://globalwordnet.github.io/schemas/dc/">
  <Lexicon id="ewn" label="English" language="en" version="2025">
    <LexicalEntry id="e-covered-a">
      <Lemma writtenForm="covered" partOfSpeech="a"/>
      <Sense id="s-covered-a-1" synset="s-covered"/>
    </LexicalEntry>
    <LexicalEntry id="e-employer-n">
      <Lemma writtenForm="employer" partOfSpeech="n"/>
      <Sense id="s-employer-n-1" synset="s-employer"/>
    </LexicalEntry>
    <LexicalEntry id="e-statute-n">
      <Lemma writtenForm="statute" partOfSpeech="n"/>
      <Sense id="s-statute-n-1" synset="s-statute"/>
    </LexicalEntry>
    <LexicalEntry id="e-retaliation-n">
      <Lemma writtenForm="retaliation" partOfSpeech="n"/>
      <Sense id="s-retaliation-n-1" synset="s-retaliation"/>
    </LexicalEntry>
    <LexicalEntry id="e-discrimination-n">
      <Lemma writtenForm="discrimination" partOfSpeech="n"/>
      <Sense id="s-discrimination-n-1" synset="s-discrimination"/>
    </LexicalEntry>
    <LexicalEntry id="e-court-n">
      <Lemma writtenForm="court" partOfSpeech="n"/>
      <Sense id="s-court-n-1" synset="s-court-legal"/>
      <Sense id="s-court-n-2" synset="s-court-yard"/>
    </LexicalEntry>
    <Synset id="s-covered" ili="i1" partOfSpeech="a" members="e-covered-a">
      <Definition>protected by or under coverage</Definition>
    </Synset>
    <Synset id="s-employer" ili="i2" partOfSpeech="n" members="e-employer-n">
      <Definition>one who employs others</Definition>
    </Synset>
    <Synset id="s-statute" ili="i3" partOfSpeech="n" members="e-statute-n">
      <Definition>a law enacted by a legislature</Definition>
    </Synset>
    <Synset id="s-retaliation" ili="i4" partOfSpeech="n" members="e-retaliation-n">
      <Definition>action taken in return for an injury or offense</Definition>
    </Synset>
    <Synset id="s-discrimination" ili="i5" partOfSpeech="n" members="e-discrimination-n">
      <Definition>unjust treatment of categories of people</Definition>
    </Synset>
    <Synset id="s-court-legal" ili="i6" partOfSpeech="n" members="e-court-n">
      <Definition>an assembly for the administration of justice</Definition>
    </Synset>
    <Synset id="s-court-yard" ili="i7" partOfSpeech="n" members="e-court-n">
      <Definition>an open uncovered space around a building</Definition>
    </Synset>
  </Lexicon>
</LexicalResource>"#;

    fn sample_english() -> English {
        let wn = lmf::reader::read_wordnet(SAMPLE_LMF).expect("sample WordNet parses");
        English::from_wordnet(&wn)
    }

    fn en_form(word: &str) -> Form {
        Form {
            written_rep: word.to_string(),
            lang: "en".to_string(),
        }
    }

    // ── resolve_form_to_senses ───────────────────────────────────────

    #[test]
    fn resolves_known_lemma_to_one_sense() {
        let en = sample_english();
        let senses = resolve_form_to_senses(&en_form("employer"), &en);
        assert_eq!(senses.len(), 1);
        assert_eq!(senses[0].reference.ontology, "english_wordnet");
        assert_eq!(senses[0].reference.concept, "s-employer");
    }

    #[test]
    fn resolves_polysemous_lemma_to_multiple_senses() {
        let en = sample_english();
        // "court" has two senses in the sample WordNet (legal +
        // courtyard).
        let senses = resolve_form_to_senses(&en_form("court"), &en);
        assert_eq!(senses.len(), 2);
        let concepts: alloc::collections::BTreeSet<String> =
            senses.iter().map(|s| s.reference.concept.clone()).collect();
        assert!(concepts.contains("s-court-legal"));
        assert!(concepts.contains("s-court-yard"));
    }

    #[test]
    fn unknown_lemma_returns_empty() {
        let en = sample_english();
        let senses = resolve_form_to_senses(&en_form("nonexistentword"), &en);
        assert!(senses.is_empty());
    }

    #[test]
    fn senses_carry_english_wordnet_ontology_tag() {
        let en = sample_english();
        let senses = resolve_form_to_senses(&en_form("statute"), &en);
        assert_eq!(senses.len(), 1);
        assert_eq!(senses[0].reference.ontology, "english_wordnet");
    }

    // ── resolve_lemmas_to_senses ─────────────────────────────────────

    #[test]
    fn resolves_multiple_forms() {
        let en = sample_english();
        let forms = vec![en_form("statute"), en_form("retaliation")];
        let mappings = resolve_lemmas_to_senses(&forms, &en);
        assert_eq!(mappings.len(), 2);
        assert!(mappings[0].is_resolved());
        assert!(mappings[1].is_resolved());
    }

    #[test]
    fn preserves_form_in_mapping() {
        let en = sample_english();
        let forms = vec![en_form("employer")];
        let mappings = resolve_lemmas_to_senses(&forms, &en);
        assert_eq!(mappings[0].form, en_form("employer"));
    }

    #[test]
    fn unknown_form_is_unresolved_but_present() {
        let en = sample_english();
        let forms = vec![en_form("nonexistent")];
        let mappings = resolve_lemmas_to_senses(&forms, &en);
        assert_eq!(mappings.len(), 1);
        assert!(!mappings[0].is_resolved());
    }

    // ── resolve_term_name_to_senses (end-to-end) ─────────────────────

    #[test]
    fn end_to_end_covered_employer() {
        let en = sample_english();
        let mappings = resolve_term_name_to_senses("Covered Employer", &en);
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].form.written_rep, "covered");
        assert_eq!(mappings[1].form.written_rep, "employer");
        assert!(mappings[0].is_resolved());
        assert!(mappings[1].is_resolved());
    }

    #[test]
    fn end_to_end_filters_stopwords() {
        // "Prohibition on Retaliation" strips "on" — only
        // "prohibition" and "retaliation" reach the resolver.
        let en = sample_english();
        let mappings = resolve_term_name_to_senses("Prohibition on Retaliation", &en);
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].form.written_rep, "prohibition");
        assert_eq!(mappings[1].form.written_rep, "retaliation");
        // "prohibition" isn't in our small sample WordNet — unresolved.
        assert!(!mappings[0].is_resolved());
        // "retaliation" is in the sample — resolved.
        assert!(mappings[1].is_resolved());
    }

    #[test]
    fn end_to_end_polysemy_propagates() {
        // "Court Action" → court has 2 senses, action has 0
        // (not in our sample).
        let en = sample_english();
        let mappings = resolve_term_name_to_senses("Court Action", &en);
        assert_eq!(mappings.len(), 2);
        // "court" resolves to 2 senses.
        assert_eq!(mappings[0].senses.len(), 2);
        // "action" not in our sample.
        assert_eq!(mappings[1].senses.len(), 0);
    }

    #[test]
    fn empty_term_name_yields_no_mappings() {
        let en = sample_english();
        let mappings = resolve_term_name_to_senses("", &en);
        assert!(mappings.is_empty());
    }

    #[test]
    fn print_sample_adjunction() {
        let en = sample_english();
        let term_names = [
            "Covered Employer",
            "Discrimination Prohibited",
            "Court Action",
        ];
        eprintln!("\n=== Statute → English adjunction sample ===");
        for name in &term_names {
            let mappings = resolve_term_name_to_senses(name, &en);
            eprintln!("  \"{name}\":");
            for m in mappings {
                let sense_concepts: alloc::vec::Vec<&str> = m
                    .senses
                    .iter()
                    .map(|s| s.reference.concept.as_str())
                    .collect();
                eprintln!(
                    "    {:<14} → [{}]",
                    m.form.written_rep,
                    if sense_concepts.is_empty() {
                        "(unresolved)".to_string()
                    } else {
                        sense_concepts.join(", ")
                    }
                );
            }
        }
        eprintln!();
    }
}
