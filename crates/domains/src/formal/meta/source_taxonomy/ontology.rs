//! Source taxonomy — concepts, hierarchy, adjunction graph, axioms.
//!
//! See `mod.rs` for the literature inventory and design rationale.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SourceTaxonomy",
    source: "Hart (1961) The Concept of Law, Oxford; MacLane (1971) Categories for the Working Mathematician §IV.1; Pustejovsky (1995) The Generative Lexicon, MIT Press; Vossen (1998) EuroWordNet, Springer; Sartor (2005) Legal Reasoning, Springer; Solan (1993) The Language of Judges, Univ. Chicago; Marbury v. Madison, 5 U.S. 137 (1803); Wilkinson et al. (2016) FAIR Guiding Principles, Scientific Data 3:160018; Dolstra (2006) The Purely Functional Software Deployment Model, PhD thesis Utrecht University",

    concepts: [
        // === Root ===
        Source,

        // === Lexicon family ===
        Lexicon,
        Language,            // Lexicon for a natural language (English WordNet, etc.)
        DomainLexicon,       // Lexicon restricted to a specialty
        LegalLexicon,        // DomainLexicon for legal terms of art (Black's, statutory defs)

        // === LegalCorpus family (Hart 1961 primary + secondary rules) ===
        LegalCorpus,
        Statute,                // primary rule (legislative enactment)
        Regulation,             // secondary rule (agency-promulgated, implements statute)
        ConstitutionalArticle,  // supreme primary rule (Marbury v. Madison 1803)
        ProceduralRule,         // primary rule of procedure (FRCP, OALJ rules)
        CaseLaw,                // secondary rule (judicial precedent)
    ],

    labels: {
        Source: ("en", "Source",
            "Wilkinson et al. (2016) FAIR F1: the abstract root of every external corpus praxis can ingest."),
        Lexicon: ("en", "Lexicon",
            "Pustejovsky (1995) The Generative Lexicon: a structured lexical resource pairing entries with senses and qualia."),
        Language: ("en", "Language",
            "Vossen (1998) EuroWordNet: a Lexicon for a natural language (English, Spanish, etc.) — the bridge for general-vocabulary anchoring."),
        DomainLexicon: ("en", "Domain lexicon",
            "Pustejovsky (1995): a Lexicon scoped to a specialty domain, with domain-specific qualia (e.g., legal, medical, scientific)."),
        LegalLexicon: ("en", "Legal lexicon",
            "Solan (1993) The Language of Judges: a DomainLexicon for legal terms of art — statutory definitions, Black's Law Dictionary, judicial glossaries."),
        LegalCorpus: ("en", "Legal corpus",
            "Hart (1961) The Concept of Law: the root of legal text resources — primary rules (statutes, constitutional articles, procedural rules) and secondary rules (regulations, case law) about them."),
        Statute: ("en", "Statute",
            "Hart (1961) primary rule: a legislative enactment binding within its jurisdiction (e.g., 18 U.S.C. § 1514A)."),
        Regulation: ("en", "Regulation",
            "Hart (1961) secondary rule: an agency-promulgated rule implementing a statute (e.g., 29 CFR Part 1980)."),
        ConstitutionalArticle: ("en", "Constitutional article",
            "Marbury v. Madison (1803): the supreme primary rule, authorizing legislation and judicial review."),
        ProceduralRule: ("en", "Procedural rule",
            "Sartor (2005): a primary rule governing court procedure (FRCP, FRE, OALJ rules)."),
        CaseLaw: ("en", "Case law",
            "Sartor (2005): a secondary, interpretive rule emerging from judicial decisions; binds via stare decisis."),
    },

    is_a: [
        // Lexicon family
        (Lexicon, Source),
        (Language, Lexicon),
        (DomainLexicon, Lexicon),
        (LegalLexicon, DomainLexicon),

        // LegalCorpus family
        (LegalCorpus, Source),
        (Statute, LegalCorpus),
        (Regulation, LegalCorpus),
        (ConstitutionalArticle, LegalCorpus),
        (ProceduralRule, LegalCorpus),
        (CaseLaw, LegalCorpus),
    ],

    // Adjunction graph: pairs of concepts whose instances are connected by
    // adjoint functor pairs. The codegen reads these at build time and
    // emits a `<AName>To<BName>` adjunction for every pair of loaded
    // instances `(a, b)` where `a` inhabits the source concept and `b`
    // inhabits the target concept. MacLane (1971) §IV.1 grounds the
    // adjoint-pair semantics; each edge below names a domain-specific
    // adjunction with its own literature pointer.
    edges: [
        // Hart (1961) §V: statute authorizes regulation; regulation
        // implements statute. The unit/counit pair surfaces "statute
        // provisions without implementing regs" and "regs without
        // statutory basis" as defensible gaps.
        (Statute, Regulation, Adjoins),

        // Solan (1993): statute terms of art anchored in the legal
        // lexicon.
        (Statute, LegalLexicon, Adjoins),

        // Sartor (2005): statutes reference procedure (e.g., a
        // whistleblower statute's exhaustion requirement points into
        // the procedural code).
        (Statute, ProceduralRule, Adjoins),

        // Regulations reuse the same terms of art the statutes define.
        (Regulation, LegalLexicon, Adjoins),

        // Sartor (2005): judicial precedent interprets primary rules and
        // their implementing regs.
        (CaseLaw, Statute, Adjoins),
        (CaseLaw, Regulation, Adjoins),
        (CaseLaw, LegalLexicon, Adjoins),

        // Marbury v. Madison (1803): constitution authorizes statutes
        // and judicial review of them.
        (ConstitutionalArticle, Statute, Adjoins),
        (ConstitutionalArticle, CaseLaw, Adjoins),

        // Procedural rules carry their own terminology.
        (ProceduralRule, LegalLexicon, Adjoins),

        // Solan (1993): legal English bridges to common English —
        // every legal-corpus chain reaches Language transitively
        // through this edge.
        (LegalLexicon, Language, Adjoins),
    ],
}

// ---------------------------------------------------------------------------
// String <-> Concept conversion (parser boundary)
// ---------------------------------------------------------------------------
//
// TOML carries `type = "Statute"` as a string; the parser maps that string
// directly to a `SourceTaxonomyConcept` variant, so every downstream call
// site is typed. Unknown names fail closed — no silent default, no
// pass-through. The mapping mirrors variant identifiers exactly so the
// invariant `format!("{:?}", c)` ↔ `parse(s)` round-trips.

/// Parse a praxis-taxonomy concept name into its typed variant. `None` if
/// `s` does not match any declared concept.
pub fn parse_concept(s: &str) -> Option<SourceTaxonomyConcept> {
    use SourceTaxonomyConcept as C;
    Some(match s {
        "Source" => C::Source,
        "Lexicon" => C::Lexicon,
        "Language" => C::Language,
        "DomainLexicon" => C::DomainLexicon,
        "LegalLexicon" => C::LegalLexicon,
        "LegalCorpus" => C::LegalCorpus,
        "Statute" => C::Statute,
        "Regulation" => C::Regulation,
        "ConstitutionalArticle" => C::ConstitutionalArticle,
        "ProceduralRule" => C::ProceduralRule,
        "CaseLaw" => C::CaseLaw,
        _ => return None,
    })
}

/// Canonical string for a concept. Used in error messages and as the
/// inverse of [`parse_concept`].
pub fn concept_name(c: SourceTaxonomyConcept) -> &'static str {
    use SourceTaxonomyConcept as C;
    match c {
        C::Source => "Source",
        C::Lexicon => "Lexicon",
        C::Language => "Language",
        C::DomainLexicon => "DomainLexicon",
        C::LegalLexicon => "LegalLexicon",
        C::LegalCorpus => "LegalCorpus",
        C::Statute => "Statute",
        C::Regulation => "Regulation",
        C::ConstitutionalArticle => "ConstitutionalArticle",
        C::ProceduralRule => "ProceduralRule",
        C::CaseLaw => "CaseLaw",
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk `is_a` to find every ancestor of `concept` in the taxonomy. Returns
/// the ancestors in any order; does not include `concept` itself.
pub fn ancestors_of(concept: SourceTaxonomyConcept) -> Vec<SourceTaxonomyConcept> {
    let sub: Vec<_> = SourceTaxonomyCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == SourceTaxonomyRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    let mut out = Vec::new();
    let mut stack = vec![concept];
    while let Some(c) = stack.pop() {
        for (s, t) in &sub {
            if *s == c && !out.contains(t) {
                out.push(*t);
                stack.push(*t);
            }
        }
    }
    out
}

/// True iff `concept` is in the LegalCorpus subtree (i.e., LegalCorpus is
/// in its ancestor set).
pub fn is_legal_corpus(concept: SourceTaxonomyConcept) -> bool {
    concept == SourceTaxonomyConcept::LegalCorpus
        || ancestors_of(concept).contains(&SourceTaxonomyConcept::LegalCorpus)
}

/// True iff `concept` is in the Lexicon subtree.
pub fn is_lexicon(concept: SourceTaxonomyConcept) -> bool {
    concept == SourceTaxonomyConcept::Lexicon
        || ancestors_of(concept).contains(&SourceTaxonomyConcept::Lexicon)
}

/// True iff `concept` is a *leaf* (no proper descendant in the taxonomy).
/// The leaves are the kinds a `[[source]]` entry can declare as its
/// `type` field.
pub fn is_leaf(concept: SourceTaxonomyConcept) -> bool {
    use SourceTaxonomyConcept as C;
    matches!(
        concept,
        C::Language
            | C::LegalLexicon
            | C::Statute
            | C::Regulation
            | C::ConstitutionalArticle
            | C::ProceduralRule
            | C::CaseLaw
    )
}

/// Adjunction edges from this concept (the right-hand sides of `Adjoins`
/// edges with `concept` as the source). The codegen consults this to
/// emit per-instance adjunction functors automatically.
pub fn adjoint_targets(concept: SourceTaxonomyConcept) -> Vec<SourceTaxonomyConcept> {
    SourceTaxonomyCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == SourceTaxonomyRelationKind::Adjoins && m.source() == concept)
        .map(|m| m.target())
        .collect()
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Hart (1961) primary-vs-secondary distinction.
///
/// Primary rules directly govern conduct (statutes, constitutional
/// articles, procedural rules). Secondary rules are *about* primary
/// rules — how they are recognized, changed, adjudicated (regulations
/// implement statutes; case law interprets them; legal lexicons gloss
/// their terms).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HartRuleKind {
    Primary,
    Secondary,
    NotApplicable,
}

#[derive(Debug, Clone)]
pub struct HartRule;

impl Quality for HartRule {
    type Individual = SourceTaxonomyConcept;
    type Value = HartRuleKind;

    fn get(&self, concept: &SourceTaxonomyConcept) -> Option<HartRuleKind> {
        use SourceTaxonomyConcept as C;
        Some(match concept {
            C::Statute | C::ConstitutionalArticle | C::ProceduralRule => HartRuleKind::Primary,
            C::Regulation | C::CaseLaw | C::LegalLexicon => HartRuleKind::Secondary,
            _ => HartRuleKind::NotApplicable,
        })
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for SourceTaxonomyOntology {
    type Cat = SourceTaxonomyCategory;
    type Qual = HartRule;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SourceTaxonomyWellFormed));
        axioms.push(Box::new(EveryAdjointEdgeTyped));
        axioms.push(Box::new(LegalAdjunctionsTerminateInLanguage));
        axioms.push(Box::new(PrimarySecondaryDistinction));
        axioms
    }
}

/// Axiom: every non-root concept reaches `Source` via `is_a`.
///
/// Hart (1961) — taxonomic completeness: each kind of legal artifact
/// must inherit from a recognized category. For the broader Source
/// taxonomy, every leaf must trace back to the root or the registry
/// loses well-typed dispatch.
pub struct SourceTaxonomyWellFormed;

impl Axiom for SourceTaxonomyWellFormed {
    fn verify(&self) -> Verdict {
        for c in SourceTaxonomyConcept::variants() {
            if c == SourceTaxonomyConcept::Source {
                continue;
            }
            if !ancestors_of(c).contains(&SourceTaxonomyConcept::Source) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SourceTaxonomyWellFormed",
        "every non-root concept reaches Source via is_a",
        "Hart (1961) The Concept of Law — taxonomic completeness of legal kinds"
    );
}

pr4xis::register_axiom!(SourceTaxonomyWellFormed, "Hart (1961) The Concept of Law");

/// Axiom: every `Adjoins`-kinded edge connects two concepts in the
/// taxonomy. Trivially true at compile time because the macro
/// enforces that edges reference declared concepts, but stated
/// here so the axiom set documents the invariant explicitly.
///
/// MacLane (1971) §IV.1: an adjoint pair is by definition a pair of
/// functors between two categories. Untyped adjunctions are not well-formed.
pub struct EveryAdjointEdgeTyped;

impl Axiom for EveryAdjointEdgeTyped {
    fn verify(&self) -> Verdict {
        // The macro already enforces concept-typed edges; this axiom
        // asserts the at-least-one-Adjoins-edge invariant so an empty
        // adjunction graph would fail (no legal corpora wire up).
        let count = SourceTaxonomyCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == SourceTaxonomyRelationKind::Adjoins)
            .count();
        if count == 0 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryAdjointEdgeTyped",
        "the adjunction graph is non-empty and every edge connects declared concepts",
        "MacLane (1971) Categories for the Working Mathematician §IV.1"
    );
}

pr4xis::register_axiom!(
    EveryAdjointEdgeTyped,
    "MacLane (1971) Categories for the Working Mathematician §IV.1"
);

/// Axiom: every leaf concept in the LegalCorpus subtree reaches
/// `Language` by traversing `Adjoins` edges transitively. This is the
/// "legal text is always anchorable in natural language" invariant:
/// given a statute, you can always chain adjunctions to reach English
/// senses through some path (Statute → LegalLexicon → Language is the
/// canonical one).
///
/// Solan (1993) *The Language of Judges* — legal English is a domain
/// variant of common English; every legal-of-art term either is itself
/// a common word with a specialized sense or is glossed by terms that
/// are.
pub struct LegalAdjunctionsTerminateInLanguage;

impl Axiom for LegalAdjunctionsTerminateInLanguage {
    fn verify(&self) -> Verdict {
        let adjoins: Vec<_> = SourceTaxonomyCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == SourceTaxonomyRelationKind::Adjoins)
            .map(|m| (m.source(), m.target()))
            .collect();
        for c in SourceTaxonomyConcept::variants() {
            if !is_legal_corpus(c) || !is_leaf(c) {
                continue;
            }
            // BFS from c over Adjoins edges; must reach Language.
            let mut seen = vec![c];
            let mut stack = vec![c];
            let mut reached = false;
            while let Some(curr) = stack.pop() {
                if curr == SourceTaxonomyConcept::Language {
                    reached = true;
                    break;
                }
                for (s, t) in &adjoins {
                    if *s == curr && !seen.contains(t) {
                        seen.push(*t);
                        stack.push(*t);
                    }
                }
            }
            if !reached {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LegalAdjunctionsTerminateInLanguage",
        "every LegalCorpus leaf reaches Language via the adjunction graph",
        "Solan (1993) The Language of Judges, Univ. Chicago Press"
    );
}

pr4xis::register_axiom!(
    LegalAdjunctionsTerminateInLanguage,
    "Solan (1993) The Language of Judges, Univ. Chicago Press"
);

/// Axiom: Hart's primary-vs-secondary distinction partitions the
/// LegalCorpus leaves correctly. Statute, ConstitutionalArticle, and
/// ProceduralRule must be Primary; Regulation, CaseLaw, and
/// LegalLexicon must be Secondary.
///
/// Hart (1961) §V — the union of primary and secondary rules
/// constitutes a legal system; their distinction is what makes the
/// system intelligible.
pub struct PrimarySecondaryDistinction;

impl Axiom for PrimarySecondaryDistinction {
    fn verify(&self) -> Verdict {
        use SourceTaxonomyConcept as C;
        let q = HartRule;
        let primary = [C::Statute, C::ConstitutionalArticle, C::ProceduralRule];
        let secondary = [C::Regulation, C::CaseLaw, C::LegalLexicon];
        for c in primary {
            if q.get(&c) != Some(HartRuleKind::Primary) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        for c in secondary {
            if q.get(&c) != Some(HartRuleKind::Secondary) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PrimarySecondaryDistinction",
        "Statute/ConstitutionalArticle/ProceduralRule are Primary; Regulation/CaseLaw/LegalLexicon are Secondary",
        "Hart (1961) The Concept of Law §V"
    );
}

pr4xis::register_axiom!(
    PrimarySecondaryDistinction,
    "Hart (1961) The Concept of Law §V"
);
