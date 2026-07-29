//! Lex-specialis resolution of statutory definitions — which definition of a
//! term governs a given use (the Q2 realization, sibling of the lexical
//! sense-elevation order).
//!
//! A term like "person" may be defined at several scopes: in the enacting
//! section, for a whole title (26 U.S.C. §7701, "For purposes of this title"),
//! by the Dictionary Act for the entire U.S. Code (1 U.S.C. §1), or not at all
//! (ordinary meaning). When a use falls under more than one, the MORE SPECIFIC
//! definition displaces the more general — the general/specific canon (Scalia &
//! Garner 2012 §28). This is modelled as a defeasible PRIORITY ordering (Reiter
//! 1980; Nute's defeasible logic; Prakken & Sartor 1996), NOT disjoint
//! namespaces, because resolution is a FALL-THROUGH (section → title →
//! Dictionary Act → ordinary) and fall-through is an ordering. A separate SOFT
//! defeater carries "unless the context indicates otherwise" (1 U.S.C. §1) and
//! §7701's "manifestly incompatible" (Rowland v. California Men's Colony, 1993).
//!
//! The applicability of an `Enacted` definition is the subtree its
//! [`PinpointCite`] governs (the use lies within it iff the scope cite is a
//! prefix of the use cite), and its specificity rises with the scope's depth —
//! so a section scope outranks a title scope, which outranks the Dictionary Act,
//! which outranks ordinary meaning. The precedence is a strict partial order
//! (verified by [`DefinitionScopePrecedenceIsStrictPartialOrder`]), so a
//! well-defined governing definition always exists.
//!
//! # Literature
//!
//! - **Scalia, A. & Garner, B. A. (2012)** *Reading Law: The Interpretation of
//!   Legal Texts* §28 (the general/specific canon) — the more specific
//!   provision governs.
//! - **1 U.S.C. §1** (the Dictionary Act); **26 U.S.C. §7701** ("For purposes of
//!   this title …") — the statutory scope language.
//! - **Reiter, R. (1980)** "A Logic for Default Reasoning", *Artificial
//!   Intelligence* 13 — defaults applied unless defeated.
//! - **Prakken, H. & Sartor, G. (1996)** "A Dialectical Model of Assessing
//!   Conflicting Arguments in Legal Reasoning", *AI & Law* 4 — defeasible rule
//!   priorities (lex specialis as a priority relation).
//! - **Rowland v. California Men's Colony, 506 U.S. 194 (1993)** — the
//!   "unless the context indicates otherwise" defeater is a soft, contextual one.

#[allow(unused_imports)]
use alloc::{boxed::Box, string::String, string::ToString, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::cognitive::linguistics::lemon::lexicon::{ConceptRef, Lexicon};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::meta::identifier_format::Identifier;
use crate::social::judicial::citation::PinpointCite;
use crate::social::judicial::ontology::{LegalRelation, RelationType};
use crate::social::judicial::source_text::SourceTextRef;

/// The applicability scope of a statutory definition — the lex-specialis ladder.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefinitionScope {
    /// A definition enacted with a stated applicability subtree — the
    /// [`PinpointCite`] it governs ("in this section / chapter / title …"). It
    /// governs every use whose citation it is a prefix of; its specificity
    /// rises with the scope's depth (a section scope outranks a title scope).
    Enacted(PinpointCite),
    /// The Dictionary Act (1 U.S.C. §1) — the default for the whole U.S. Code,
    /// displaced by any `Enacted` scope that governs the use.
    DictionaryAct,
    /// No statutory definition: the ordinary lexical meaning (the general sense).
    OrdinaryMeaning,
}

impl DefinitionScope {
    /// Does this scope govern a use at `use_cite`? An `Enacted` scope governs a
    /// use iff the scope's citation is a (non-strict) prefix of the use's — the
    /// use lies within the subtree the definition applies to. The Dictionary Act
    /// and ordinary meaning govern everywhere (the fall-through floors).
    #[must_use]
    pub fn governs(&self, use_cite: &PinpointCite) -> bool {
        match self {
            DefinitionScope::Enacted(scope) => use_cite.segments.starts_with(&scope.segments),
            DefinitionScope::DictionaryAct | DefinitionScope::OrdinaryMeaning => true,
        }
    }

    /// The lex-specialis specificity — a higher value displaces a lower one
    /// (Scalia & Garner §28). An `Enacted` scope's specificity rises with its
    /// depth, always above the Dictionary-Act default, which is above ordinary
    /// meaning.
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), not a bare
    /// `u32` — a specificity rank, the same typing discipline as
    /// `formal::mereology::counting::ontology::cardinality`.
    #[must_use]
    pub fn specificity(&self) -> Quantity {
        let rank: u32 = match self {
            DefinitionScope::Enacted(cite) => 2 + cite.segments.len() as u32,
            DefinitionScope::DictionaryAct => 1,
            DefinitionScope::OrdinaryMeaning => 0,
        };
        Quantity::from_unit(rank as f64, &unit::UNITLESS)
    }

    /// Lex-specialis precedence: `self` displaces `other` for a term they both
    /// govern iff `self` is strictly more specific. A strict partial order
    /// (irreflexive, asymmetric, transitive) — see
    /// [`DefinitionScopePrecedenceIsStrictPartialOrder`].
    #[must_use]
    pub fn displaces(&self, other: &DefinitionScope) -> bool {
        self.specificity() > other.specificity()
    }
}

/// A statutory definition: a defined `term`, its applicability `scope`, and the
/// legal concept it binds the term to (`defines` — a reference into the legal
/// lexicon, the sense a use elevates to in the legal domain).
///
/// The optional `definition` carries the authoritative source text that JUSTIFIES
/// the binding — a verbatim, attributable [`SourceTextRef`] (the enacting section,
/// a glossary entry, an editorial-style guide). It is the cited, queryable
/// provenance for "why does this term mean this concept", distinct from
/// `as_defines_relation`'s structural `usc:<labels>` provenance (which names the
/// PROVISION). `None` where the layer is a hand-coded prototype that has not yet
/// attached its enacting text (the Dictionary-Act prototype below).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LegalDefinition {
    pub term: String,
    pub scope: DefinitionScope,
    pub defines: ConceptRef,
    /// The authoritative source text justifying this binding, when cited.
    pub definition: Option<SourceTextRef>,
}

impl LegalDefinition {
    /// The `Defines` morphism this definition contributes to the legal relation
    /// graph: the defining provision (source) establishes the meaning of the
    /// `term` (target). `None` for `OrdinaryMeaning` — no statutory definition,
    /// hence no morphism. Source and target are CURIE-typed [`Identifier`]s
    /// (W3C CURIE Syntax 1.0): the provision as `usc:<labels>` (the Dictionary
    /// Act is `usc:1-1`, i.e. 1 U.S.C. §1) and the term as `term:<term>`.
    pub fn as_defines_relation(&self) -> Option<LegalRelation> {
        let from_local = match &self.scope {
            DefinitionScope::Enacted(cite) => cite
                .segments
                .iter()
                .map(|s| s.label.as_str())
                .collect::<Vec<_>>()
                .join("-"),
            DefinitionScope::DictionaryAct => "1-1".to_string(),
            DefinitionScope::OrdinaryMeaning => return None,
        };
        let from = Identifier::curie(alloc::format!("usc:{from_local}")).ok()?;
        let to = Identifier::curie(alloc::format!("term:{}", self.term.replace(' ', "_"))).ok()?;
        Some(LegalRelation {
            from,
            to,
            relation: RelationType::Defines,
        })
    }
}

/// Resolve which definition of a term governs a use at `use_cite`: the MOST
/// SPECIFIC `candidate` whose scope governs the use (lex specialis), skipping
/// any the `contextual_defeater` rejects ("unless the context indicates
/// otherwise" / "manifestly incompatible", Rowland 1993) so resolution falls
/// through to the next-most-specific governing definition. `None` when nothing
/// governs (the caller falls back to ordinary meaning).
pub fn resolve_definition<'a, F>(
    use_cite: &PinpointCite,
    candidates: &'a [LegalDefinition],
    contextual_defeater: F,
) -> Option<&'a LegalDefinition>
where
    F: Fn(&LegalDefinition) -> bool,
{
    candidates
        .iter()
        .filter(|d| d.scope.governs(use_cite) && !contextual_defeater(d))
        .max_by(|a, b| {
            a.scope
                .specificity()
                .partial_cmp(&b.scope.specificity())
                .expect("specificity is UNITLESS, always comparable")
        })
}

/// The definitional layer a statute contributes — the terms it defines, each
/// with its applicability scope and the concept it binds. Title 1 (the
/// Dictionary Act) is the canonical inhabitant: it defines "person", "whoever",
/// "vessel", … for the whole U.S. Code, and every other title resolves a use of
/// those terms through it, a more specific title definition displacing it (lex
/// specialis). This is the `LegalLexicon` a `Statute` `Adjoins` in the source
/// taxonomy, made queryable.
#[derive(Debug, Clone, Default)]
pub struct DefinitionLexicon {
    definitions: Vec<LegalDefinition>,
}

impl DefinitionLexicon {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a definition this layer contributes.
    pub fn define(&mut self, definition: LegalDefinition) {
        self.definitions.push(definition);
    }

    /// The concept a use of `term` at `use_cite` resolves to — the most specific
    /// governing definition (lex specialis), skipping any the `defeater` rejects
    /// ("unless the context indicates otherwise"). `None` for ordinary meaning
    /// (no governing statutory definition).
    pub fn resolve<F>(
        &self,
        term: &str,
        use_cite: &PinpointCite,
        defeater: F,
    ) -> Option<&ConceptRef>
    where
        F: Fn(&LegalDefinition) -> bool,
    {
        self.definitions
            .iter()
            .filter(|d| d.term == term && d.scope.governs(use_cite) && !defeater(d))
            .max_by(|a, b| {
                a.scope
                    .specificity()
                    .partial_cmp(&b.scope.specificity())
                    .expect("specificity is UNITLESS, always comparable")
            })
            .map(|d| &d.defines)
    }

    /// Mint this layer's terms into `lexicon` as legal-domain senses — each
    /// defined term gains a `"legal"`-register `Sense` on its ONE shared entry,
    /// alongside whatever general sense it already carries, so a use in the legal
    /// register elevates to the statutory meaning (the sense-elevation order)
    /// while the general meaning stays reachable.
    pub fn mint_into(&self, lexicon: &mut Lexicon) {
        for d in &self.definitions {
            lexicon.add_sense(
                d.term.clone(),
                d.defines.ontology.clone(),
                d.defines.concept.clone(),
                Some("legal".to_string()),
            );
        }
    }

    /// The definitions this layer holds.
    #[must_use]
    pub fn definitions(&self) -> &[LegalDefinition] {
        &self.definitions
    }
}

/// The Dictionary Act (1 U.S.C. ch. 1, "Rules of Construction") definitional
/// layer — the terms it fixes for the whole U.S. Code "unless the context
/// indicates otherwise" (§1). Each term binds to a Title-1-native concept
/// (`usc_title_1:<term>`) at `DictionaryAct` (code-wide) scope, so a use of it
/// in any title resolves here unless a more specific title or section definition
/// displaces it (lex specialis).
///
/// This is a hand-coded PROTOTYPE — each term verbatim from its enacting section
/// — of what the corpus loader + term extraction will produce once `usc_title_1`
/// is provisioned, following the same prototype discipline the judicial
/// ontology's other primary-source concepts use. Per-term enacting-section
/// provenance (§1 vs §3/§4/§5) on the `Defines` morphism is a later refinement;
/// the resolution scope (code-wide) is already correct.
///
/// # Source
///
/// - **1 U.S.C. §1** — "person" / "whoever", "officer", "signature" /
///   "subscription", "oath" / "sworn", "writing".
/// - **1 U.S.C. §3** — "vessel"; **§4** — "vehicle"; **§5** — "company" /
///   "association".
#[must_use]
pub fn dictionary_act_definitions() -> DefinitionLexicon {
    let mut lexicon = DefinitionLexicon::new();
    for term in [
        "person",
        "whoever",
        "officer",
        "signature",
        "subscription",
        "oath",
        "sworn",
        "writing",
        "vessel",
        "vehicle",
        "company",
        "association",
    ] {
        lexicon.define(LegalDefinition {
            term: term.to_string(),
            scope: DefinitionScope::DictionaryAct,
            defines: ConceptRef {
                ontology: "usc_title_1".to_string(),
                concept: term.to_string(),
            },
            // Prototype: the per-term enacting text (§1/§3/§4/§5) is a later
            // refinement, the same prototype discipline this layer already
            // notes for its `Defines`-morphism provenance.
            definition: None,
        });
    }
    lexicon
}

/// The USLM / editorial-vocabulary definitional layer — terms the U.S. Code's
/// own markup and the Office of the Law Revision Counsel (OLRC) editors use to
/// NAME parts of a section, but which the Code does not itself define as
/// terms-of-art. "catchline" is the canonical inhabitant: in USLM and OLRC
/// practice it is the heading of a Code section.
///
/// It is grounded not in a Code definition but in an ALREADY-LOADED, cited
/// standard ontology — DoCO (the SPAR Document Components Ontology), bundled and
/// hydrated by the registry-driven `owl::loaded_vocabularies::loaded_vocabulary`
/// loader. "catchline" lexicalizes, in the legal/editorial
/// register, the DoCO class `doco:SectionTitle` (IRI
/// `http://purl.org/spar/doco/SectionTitle`; `rdfs:label` "section title";
/// `rdfs:subClassOf doco:Title`; `rdfs:comment` "The title of a section.") —
/// loaded OWL concepts are keyed by IRI, so the `defines` `ConceptRef`'s
/// `concept` is that IRI and its `ontology` is the registry name `"doco"`. The
/// authoritative DEFINITION of the concept lives on the DoCO class itself; this
/// layer only records that "catchline" is the US-Code-register word FOR it.
///
/// The scope is [`DefinitionScope::OrdinaryMeaning`]: "catchline" is editorial
/// vocabulary, NOT a Code-enacted term, so it contributes no `Defines` morphism
/// ([`LegalDefinition::as_defines_relation`] correctly yields `None`) — the word
/// is grounded by its lexicalization of a cited concept, not by a statute that
/// defines it.
///
/// # Source
///
/// - **OLRC, U.S. Code Glossary** — "Catchline: A catchline is the heading of a
///   Code section." <https://uscode.house.gov/faq.xhtml> (the mapping
///   justification, carried verbatim on the `definition` `SourceTextRef`).
/// - **DoCO `doco:SectionTitle`** — Constantin, Peroni, Pettifer, Shotton &
///   Vitali (2016) *The Document Components Ontology (DoCO)*, Semantic Web 7(2)
///   — the grounding concept (`rdfs:comment` "The title of a section.").
#[must_use]
pub fn uslm_vocabulary_definitions() -> DefinitionLexicon {
    let mut lexicon = DefinitionLexicon::new();
    lexicon.define(LegalDefinition {
        term: "catchline".to_string(),
        scope: DefinitionScope::OrdinaryMeaning,
        defines: ConceptRef {
            // Loaded OWL concepts are keyed by their IRI; `"doco"` is the
            // registry name under which DoCO is hydrated.
            ontology: "doco".to_string(),
            concept: "http://purl.org/spar/doco/SectionTitle".to_string(),
        },
        definition: Some(SourceTextRef::new(
            "Catchline: A catchline is the heading of a Code section. \
             (OLRC, U.S. Code Glossary, uscode.house.gov/faq.xhtml)",
        )),
    });
    lexicon
}

/// The lex-specialis definition-precedence order is a STRICT PARTIAL ORDER.
///
/// For a term defined at several scopes to have a well-defined governing
/// definition, the precedence `a ≺ b ⟺ a.specificity() > b.specificity()` must
/// be irreflexive, asymmetric and transitive. Since it is induced by a total
/// grading into a totally-ordered codomain it always is; this axiom discharges
/// the three laws over the four tiers a term such as "person" can carry: a
/// section-enacted definition, a title-enacted one, the Dictionary Act, and
/// ordinary meaning (Scalia & Garner §28; Prakken & Sartor 1996).
#[derive(Debug)]
pub struct DefinitionScopePrecedenceIsStrictPartialOrder;

impl Axiom for DefinitionScopePrecedenceIsStrictPartialOrder {
    fn verify(&self) -> Verdict {
        use crate::social::judicial::citation::ontology::PinpointCitationConcept as L;

        let scopes = [
            DefinitionScope::Enacted(
                PinpointCite::new()
                    .push(L::Title, "26")
                    .push(L::Section, "7701")
                    .push(L::Subsection, "a"),
            ),
            DefinitionScope::Enacted(
                PinpointCite::new()
                    .push(L::Title, "26")
                    .push(L::Section, "7701"),
            ),
            DefinitionScope::DictionaryAct,
            DefinitionScope::OrdinaryMeaning,
        ];
        for a in &scopes {
            // Irreflexive: ¬(a ≺ a).
            if a.displaces(a) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            for b in &scopes {
                // Asymmetric: a ≺ b ⟹ ¬(b ≺ a).
                if a.displaces(b) && b.displaces(a) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                for c in &scopes {
                    // Transitive: a ≺ b ∧ b ≺ c ⟹ a ≺ c.
                    if a.displaces(b) && b.displaces(c) && !a.displaces(c) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DefinitionScopePrecedenceIsStrictPartialOrder",
        "the lex-specialis definition-precedence order (more specific displaces more general) is a strict partial order (irreflexive, asymmetric, transitive)",
        "Scalia & Garner (2012) Reading Law §28; 1 U.S.C. §1; 26 U.S.C. §7701; Reiter (1980) AIJ 13; Prakken & Sartor (1996) AI & Law 4"
    );
}
pr4xis::register_axiom!(
    DefinitionScopePrecedenceIsStrictPartialOrder,
    "Scalia & Garner (2012) Reading Law §28; Reiter (1980) AIJ 13; Prakken & Sartor (1996) AI & Law 4"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::judicial::citation::ontology::PinpointCitationConcept as L;

    fn cite(parts: &[(L, &str)]) -> PinpointCite {
        let mut c = PinpointCite::new();
        for (lvl, lbl) in parts {
            c = c.push(*lvl, *lbl);
        }
        c
    }

    fn person(scope: DefinitionScope) -> LegalDefinition {
        LegalDefinition {
            term: "person".to_string(),
            scope,
            defines: ConceptRef {
                ontology: "us_legal_lexicon".to_string(),
                concept: "person".to_string(),
            },
            definition: None,
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn precedence_axiom_holds() {
        assert!(
            DefinitionScopePrecedenceIsStrictPartialOrder
                .verify()
                .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn more_specific_scope_displaces_more_general() {
        let title = DefinitionScope::Enacted(cite(&[(L::Title, "26"), (L::Section, "7701")]));
        let section = DefinitionScope::Enacted(cite(&[
            (L::Title, "26"),
            (L::Section, "7701"),
            (L::Subsection, "a"),
        ]));
        assert!(section.displaces(&title));
        assert!(title.displaces(&DefinitionScope::DictionaryAct));
        assert!(DefinitionScope::DictionaryAct.displaces(&DefinitionScope::OrdinaryMeaning));
        // …and the order does NOT run the other way.
        assert!(!title.displaces(&section));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn governs_by_containment() {
        let title_def = DefinitionScope::Enacted(cite(&[(L::Title, "26")]));
        // A use anywhere within Title 26 is governed…
        assert!(title_def.governs(&cite(&[(L::Title, "26"), (L::Section, "61")])));
        // …a use in Title 18 is not.
        assert!(!title_def.governs(&cite(&[(L::Title, "18"), (L::Section, "1")])));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn resolve_picks_most_specific_governing_definition() {
        let use_cite = cite(&[(L::Title, "26"), (L::Section, "7701"), (L::Subsection, "a")]);
        let candidates = [
            person(DefinitionScope::DictionaryAct),
            person(DefinitionScope::Enacted(cite(&[
                (L::Title, "26"),
                (L::Section, "7701"),
            ]))),
        ];
        let chosen = resolve_definition(&use_cite, &candidates, |_| false).expect("a definition");
        assert!(
            matches!(chosen.scope, DefinitionScope::Enacted(_)),
            "the title definition displaces the Dictionary Act"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn contextual_defeater_falls_through_to_next() {
        // "unless the context indicates otherwise": defeat the title definition,
        // and the Dictionary Act governs instead.
        let use_cite = cite(&[(L::Title, "26"), (L::Section, "7701")]);
        let title_def = person(DefinitionScope::Enacted(cite(&[
            (L::Title, "26"),
            (L::Section, "7701"),
        ])));
        let candidates = [person(DefinitionScope::DictionaryAct), title_def.clone()];
        let chosen =
            resolve_definition(&use_cite, &candidates, |d| *d == title_def).expect("a definition");
        assert!(
            matches!(chosen.scope, DefinitionScope::DictionaryAct),
            "the defeated title definition falls through to the Dictionary Act"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn definition_lexicon_resolves_lex_specialis_and_mints_legal_sense() {
        use crate::cognitive::linguistics::lemon::lexicon::Lexicon;

        // Title 1 (the Dictionary Act) defines "person" code-wide; Title 26
        // §7701 redefines it for its own title.
        let mut layer = DefinitionLexicon::new();
        layer.define(LegalDefinition {
            term: "person".to_string(),
            scope: DefinitionScope::DictionaryAct,
            defines: ConceptRef {
                ontology: "us_legal_lexicon".to_string(),
                concept: "dictionary_act_person".to_string(),
            },
            definition: None,
        });
        layer.define(LegalDefinition {
            term: "person".to_string(),
            scope: DefinitionScope::Enacted(cite(&[(L::Title, "26"), (L::Section, "7701")])),
            defines: ConceptRef {
                ontology: "us_legal_lexicon".to_string(),
                concept: "title26_person".to_string(),
            },
            definition: None,
        });

        // A use within Title 26 §7701 resolves to the title-specific definition…
        let in_t26 = cite(&[(L::Title, "26"), (L::Section, "7701"), (L::Subsection, "a")]);
        assert_eq!(
            layer.resolve("person", &in_t26, |_| false).unwrap().concept,
            "title26_person"
        );
        // …a use elsewhere falls through to the Dictionary Act.
        let in_t18 = cite(&[(L::Title, "18"), (L::Section, "1")]);
        assert_eq!(
            layer.resolve("person", &in_t18, |_| false).unwrap().concept,
            "dictionary_act_person"
        );

        // Minting puts the legal senses onto the shared lexicon atom, alongside
        // the general sense, and the legal register elevates to a legal sense.
        let mut lex = Lexicon::new("en");
        lex.add_sense("person", "english_wordnet", "person.n.01", None);
        layer.mint_into(&mut lex);
        assert!(lex.lookup("person").unwrap().senses.len() >= 2);
        let legal = lex.resolve("person", Some("legal")).expect("a sense");
        assert_eq!(legal.reference.ontology, "us_legal_lexicon");
        // The general sense is still reachable by default.
        assert_eq!(
            lex.resolve("person", None).unwrap().reference.ontology,
            "english_wordnet"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn dictionary_act_definition_yields_a_defines_morphism() {
        let def = person(DefinitionScope::DictionaryAct);
        let rel = def.as_defines_relation().expect("a Defines morphism");
        assert_eq!(rel.relation, RelationType::Defines);
        assert_eq!(rel.from.value(), "usc:1-1"); // 1 U.S.C. §1, the Dictionary Act
        assert_eq!(rel.to.value(), "term:person");
        // An Enacted definition names its provision; ordinary meaning yields none.
        let enacted = person(DefinitionScope::Enacted(cite(&[
            (L::Title, "26"),
            (L::Section, "7701"),
        ])));
        assert_eq!(
            enacted.as_defines_relation().unwrap().from.value(),
            "usc:26-7701"
        );
        assert!(
            person(DefinitionScope::OrdinaryMeaning)
                .as_defines_relation()
                .is_none()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn dictionary_act_makes_title_1_definitions_resolvable() {
        let title1 = dictionary_act_definitions();
        assert_eq!(title1.definitions().len(), 12);

        // A use of "person" anywhere in the Code resolves to the Dictionary Act
        // definition (no more-specific title definition in play).
        let use_in_t18 = cite(&[(L::Title, "18"), (L::Section, "1")]);
        let person_def = title1
            .resolve("person", &use_in_t18, |_| false)
            .expect("a definition");
        assert_eq!(person_def.ontology, "usc_title_1");
        assert_eq!(person_def.concept, "person");

        // Minting Title 1 onto a lexicon: "person" gains its legal sense beside
        // its general WordNet sense; the legal register elevates to Title 1's,
        // the default stays WordNet's — and "vessel" is now a Title-1 term too.
        let mut lex = Lexicon::new("en");
        lex.add_sense("person", "english_wordnet", "person.n.01", None);
        title1.mint_into(&mut lex);
        assert_eq!(
            lex.resolve("person", Some("legal"))
                .unwrap()
                .reference
                .ontology,
            "usc_title_1"
        );
        assert_eq!(
            lex.resolve("person", None).unwrap().reference.ontology,
            "english_wordnet"
        );
        assert!(lex.lookup("vessel").is_some());
    }

    /// (a) RESOLUTION — the praxis-way crux: the concept "catchline" grounds in
    /// is NOT a dangling reference. The `defines` `ConceptRef` names the loaded
    /// DoCO vocabulary by registry key (`"doco"`) and the concept by IRI; this
    /// genuinely hydrates DoCO and resolves that IRI to a real loaded
    /// `owl:Class` through the typed [`LoadedOwlVocabulary`] accessors — not a
    /// `String==` on the IRI, not a graceful skip. (The sibling
    /// `loaded_vocabulary_resolves_cito_subproperty` proves `loaded_vocabulary`
    /// hydrates SPAR vocabularies in this same lib suite, so DoCO resolves too.)
    ///
    /// Gated on `feature = "fetch"` exactly like the sibling
    /// `loaded_vocabulary_resolves_cito_subproperty`: the `loaded_vocabularies`
    /// module that hydrates DoCO is itself `#[cfg(all(feature = "fetch", …))]`
    /// (it needs `prx`'s `build_envelope` + the codegen materialiser), so this
    /// resolution proof runs whenever that loader is compiled — `cargo test
    /// --features fetch` here, the `fetch`-enabled CI test job there.
    #[cfg(feature = "fetch")]
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn catchline_grounds_in_loaded_doco_section_title() {
        use crate::social::software::markup::xml::owl::loaded_vocabularies::loaded_vocabulary;
        use crate::social::software::markup::xml::owl::vocabulary::OwlEntityKind;

        let layer = uslm_vocabulary_definitions();
        let catchline = layer
            .definitions()
            .iter()
            .find(|d| d.term == "catchline")
            .expect("uslm layer defines catchline");

        // The grounding target, read off the definition itself (no literal).
        let ConceptRef { ontology, concept } = &catchline.defines;
        assert_eq!(ontology, "doco", "catchline grounds in the DoCO vocabulary");

        // DoCO is a registered, bundled OntologyVocabulary — it must hydrate.
        let doco = loaded_vocabulary(ontology)
            .expect("doco must be a registered, on-disk OntologyVocabulary");

        // The IRI resolves to a REAL loaded concept (not a dangling ref).
        let idx = doco
            .find(concept)
            .unwrap_or_else(|| panic!("catchline's grounding IRI {concept} must resolve in DoCO"));
        let entity = doco.entity(idx).expect("resolved index is in range");
        assert_eq!(
            entity.kind,
            OwlEntityKind::Class,
            "doco:SectionTitle is an owl:Class"
        );
        // …and it carries DoCO's own authoritative label + comment.
        assert_eq!(
            doco.label_of(concept),
            Some("section title"),
            "doco:SectionTitle's rdfs:label"
        );
        assert_eq!(
            doco.definition_of(concept),
            Some("The title of a section."),
            "doco:SectionTitle's rdfs:comment is the authoritative definition"
        );
        // It is a kind of doco:Title (the modeling claim "a catchline is a
        // section's title" holds in the loaded taxonomy, strict is_a).
        assert!(
            doco.is_a(concept, "http://purl.org/spar/doco/Title"),
            "doco:SectionTitle rdfs:subClassOf doco:Title"
        );
    }

    /// (b) MINT: `uslm_vocabulary_definitions().mint_into(lexicon)` puts a
    /// `"legal"`-register sense for "catchline" on the lexicon, whose reference
    /// is exactly the loaded DoCO concept — the word is now understood as a
    /// lexicalization of `doco:SectionTitle`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn catchline_mints_a_sense_referencing_loaded_doco_concept() {
        use crate::cognitive::linguistics::lemon::lexicon::Lexicon;

        let mut lex = Lexicon::new("en");
        uslm_vocabulary_definitions().mint_into(&mut lex);

        let entry = lex.lookup("catchline").expect("catchline entry minted");
        let want = ConceptRef {
            ontology: "doco".to_string(),
            concept: "http://purl.org/spar/doco/SectionTitle".to_string(),
        };
        assert!(
            entry.senses.iter().any(|s| s.reference == want),
            "catchline carries a Sense referencing doco:SectionTitle, got {:?}",
            entry.senses
        );
        // The minted sense is in the legal/editorial register.
        let legal = lex
            .resolve("catchline", Some("legal"))
            .expect("a legal sense");
        assert_eq!(legal.reference, want);
    }

    /// The catchline definition is `OrdinaryMeaning` scope — editorial
    /// vocabulary, not a Code-enacted term — so it contributes NO `Defines`
    /// morphism, and it carries its OLRC-glossary justification verbatim on the
    /// structured `definition` `SourceTextRef`.
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn catchline_yields_no_defines_morphism_but_cites_its_source() {
        let layer = uslm_vocabulary_definitions();
        let catchline = layer
            .definitions()
            .iter()
            .find(|d| d.term == "catchline")
            .expect("catchline defined");
        assert_eq!(catchline.scope, DefinitionScope::OrdinaryMeaning);
        assert!(
            catchline.as_defines_relation().is_none(),
            "editorial vocabulary is not a Code-defined term, so no Defines morphism"
        );
        let cite = catchline
            .definition
            .as_ref()
            .expect("catchline cites the OLRC glossary");
        assert!(
            cite.as_str().contains("heading of a Code section"),
            "the OLRC glossary justification is carried verbatim"
        );
    }
}
