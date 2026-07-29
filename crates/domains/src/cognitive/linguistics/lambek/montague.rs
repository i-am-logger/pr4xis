#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis_runtime::ontology::ConceptRef;

use super::reduce::{ExpressionUse, TokenSpan, TypedToken};
use super::types::LambekType;
use crate::cognitive::linguistics::english::{ConceptId, LexicalReasoner};

// The Montague functor: Syntax → Semantics.
//
// Type-driven interpretation: each Lambek type maps to a semantic domain.
// The mapping IS a functor — composition in syntax maps to composition in semantics.
//
// Atomic types → Semantic domains:
//   NP → Entity (a reference to a thing)
//   S  → Proposition (a truth-evaluable statement)
//   N  → Predicate (a property: λx.dog(x))
//
// Complex types → Function spaces:
//   A/B → (B-domain → A-domain)
//   A\B → (A-domain → B-domain)
//
// Reduction → Function application:
//   (A/B) + B → A  ≡  f(x) where f: B→A, x: B
//
// References:
// - Montague, The Proper Treatment of Quantification (1973)
// - Coecke, Sadrzadeh, Clark, DisCoCat (2010)

/// The grammatical role a constituent plays — Steedman (2000) *The Syntactic
/// Process*, the argument/functor distinction. A saturated (atom-typed) category
/// — `NP`/`N`/`PP`/`S` — is an [`Argument`](GrammaticalRole::Argument): it can be
/// one of the queried entities. A slash category (`A/B`, `A\B`) is a
/// [`Functor`](GrammaticalRole::Functor): it is *syncategorematic* — it
/// contributes a function, never itself a queried NP. The copula `(S\NP)/NP` /
/// question-copula `(S[q]/NP)/NP` is therefore a `Functor` — Bowers (1993) *The
/// Syntax of Predication* (LI 24) and Pollard & Sag (1994) *Head-Driven Phrase
/// Structure Grammar* analyse the copula as a non-θ-assigning linking verb, never
/// one of the queried entities. This is the parse-time datum that decides "a
/// queried entity", replacing lexicon-membership (WordNet lookup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammaticalRole {
    /// A saturated (atom-typed) constituent — a subject or predicative-nominal
    /// complement. One of the queried entities of the question.
    Argument,
    /// A slash-typed (functor) constituent — syncategorematic (the copula). It
    /// contributes a function, never a queried entity.
    Functor,
}

impl GrammaticalRole {
    /// The role a Lambek type assigns — Steedman (2000)'s syncategorematic
    /// criterion made operational: an atom is an [`Argument`](Self::Argument), a
    /// slash (functor) category is a [`Functor`](Self::Functor).
    pub fn from_lambek_type(ty: &LambekType) -> Self {
        match ty {
            LambekType::Atom(_) => Self::Argument,
            LambekType::RightDiv(_, _) | LambekType::LeftDiv(_, _) => Self::Functor,
        }
    }
}

/// Where a predicate's SURFACE came from — uttered as a lexical token, or
/// MINTED by semantic composition (the N-result modifier+head concatenation in
/// the composition rule's `apply`). The distinction is load-bearing for the answer layer: a
/// definiendum is a lexical unit — dictionaries define headword lemmas, never
/// derivation-minted strings (Atkins & Rundell 2008, *The Oxford Guide to
/// Practical Lexicography*, headword/lemma selection; Quine 1940:26 via SEP
/// "Quotation" §3.1: a mentioned/queried expression is a singular term). So an
/// unresolvable [`Composite`](PredProvenance::Composite) surface (the
/// degenerate `is:N/N + a:N → "is a"` derivation of "what is a long") is never
/// presented as an unknown *word*; its failing LEAF constituents are the
/// reportable unresolved surfaces. A RESOLVING composite (the attributive
/// "dormant fault") is unaffected — provenance only routes the failure case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredProvenance {
    /// The surface is a token as uttered (or a collapsed loaded surface).
    Lexical,
    /// The surface was concatenated by the N-result composition rule from a
    /// functor word and an argument word (`"{func} {arg}"`).
    Composite {
        /// The modifier/functor half of the concatenation (`"is"` / `"dormant"`).
        func: String,
        /// The head/argument half of the concatenation (`"a"` / `"fault"`).
        arg: String,
    },
}

/// The illocutionary TYPE of a question — Huddleston & Pullum (2002) *The
/// Cambridge Grammar of the English Language* §10.2: a closed (polar)
/// interrogative asks for a truth value ("is a dog a mammal?" — yes/no); an
/// open (constituent) interrogative asks for a value filling a wh-gap ("what
/// is a dog?"). Read directly off the CCG `S[q]`/`S[wq]` feature (Steedman
/// 2000) at composition time — carried on [`Sem::Question`] rather than
/// re-derived downstream from entity count or predicate text, which is
/// exactly the confusion that let a degenerate polar question (a failed
/// second-argument extraction, leaving only one resolved entity) fall through
/// to the single-entity wh-question define path and answer with the wrong
/// illocutionary force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionIllocution {
    /// `S[q]` — "is a dog a mammal?": asks for a truth value.
    Polar,
    /// `S[wq]` — "what is a dog?": asks for a value filling the wh-gap.
    Content,
}

/// A semantic value — lives in the semantic domain determined by its Lambek type.
#[derive(Debug, Clone, PartialEq)]
pub enum Sem {
    /// Entity domain (NP): a reference to something in the world — or, when
    /// [`expression_use`](Sem::Concept::expression_use) is
    /// [`Mentioned`](ExpressionUse::Mentioned), to an EXPRESSION.
    Concept {
        word: String,
        concepts: Vec<ConceptId>,
        /// The parse-time grammatical role (Steedman 2000) — an NP entity is an
        /// [`Argument`](GrammaticalRole::Argument).
        role: GrammaticalRole,
        /// WHAT this entity refers to: its ordinary referent
        /// ([`Used`](ExpressionUse::Used)) or the written expression itself
        /// ([`Mentioned`](ExpressionUse::Mentioned)) — carried up from the
        /// token ([`TypedToken::expression_use`]), never re-derived.
        ///
        /// Load-bearing, not decorative: a MENTIONED concept is the ONLY
        /// shape a definiendum can take (`the term “X” means …` /
        /// `“X” means …` — the definiendum is talked ABOUT, the definiens is
        /// used), so `statute_structure::grounding::definiendum_words` reads
        /// this field to tell a real definiendum from an ordinary subject
        /// NP that happens to sit in the same argument slot. Before this
        /// field existed the two were indistinguishable — the very
        /// indistinguishability that function's own doc recorded as the
        /// reason its COORDINATED branch needed a reserved marker — and
        /// `defines` edges were minted for used subjects ("words",
        /// "provided", "may").
        ///
        /// A mentioned concept carries NO `concepts`: WordNet senses are the
        /// referents of a USED word, and attaching the physical phenomenon
        /// `radiation` to the SUBJECT of `“radiation” means ionizing …
        /// radiation` states the wrong proposition (Quine 1940 §4 — see
        /// [`ExpressionUse`]).
        expression_use: ExpressionUse,
    },
    /// Predicate domain (N): a property that can be true of entities.
    Pred {
        word: String,
        /// The parse-time grammatical role (Steedman 2000).
        role: GrammaticalRole,
        /// Lexical token or composition-minted concatenation — see
        /// [`PredProvenance`].
        provenance: PredProvenance,
    },
    /// Proposition domain (S): a complete truth-evaluable statement.
    Prop {
        predicate: String,
        arguments: Vec<Sem>,
    },
    /// Question domain (Q): a proposition that asks for truth value or information.
    Question {
        predicate: String,
        arguments: Vec<Sem>,
        /// Which of the two question types this was parsed as — read directly
        /// off the CCG `S[q]`/`S[wq]` feature at composition time. See
        /// [`QuestionIllocution`].
        illocution: QuestionIllocution,
    },
    /// Function domain (A/B or A\B): a function waiting for an argument.
    ///
    /// `body` accumulates EVERY argument absorbed so far, in absorption
    /// order — empty at first lexing (`lex`'s function-domain branch), one
    /// element after the first partial application, two after the second,
    /// and so on. A function that needs 2+ arguments before reaching an
    /// atomic (S/NP/N) result — a ditransitive verb, or a do-support chain
    /// like "does X mean" (subject NP absorbed first, then the bare verb) —
    /// passes through the function-result branch of the composition rule's
    /// `apply` more than once; a single `Box<Sem>` there silently overwrote
    /// the FIRST absorbed argument with the second (confirmed defect, fixed
    /// by accumulating into this `Vec` instead of replacing a single slot).
    Func { word: String, body: Vec<Sem> },
}

impl Sem {
    pub fn describe(&self) -> String {
        match self {
            Sem::Concept { word, .. } => word.clone(),
            Sem::Pred { word, .. } => format!("λx.{}(x)", word),
            Sem::Prop {
                predicate,
                arguments,
            } => {
                let args: Vec<String> = arguments.iter().map(|a| a.describe()).collect();
                format!("{}({})", predicate, args.join(", "))
            }
            Sem::Question {
                predicate,
                arguments,
                ..
            } => {
                let args: Vec<String> = arguments.iter().map(|a| a.describe()).collect();
                format!("?{}({})", predicate, args.join(", "))
            }
            Sem::Func { word, .. } => format!("λ.{}", word),
        }
    }

    /// The honest "no derivation" sentinel — `interpret`'s own placeholder
    /// for "nothing composed a complete meaning," exposed so callers that
    /// already know composition is not going to be trustworthy (the syntax
    /// chart itself failed to find a parse) can produce it directly instead
    /// of running the CYK search over unreliable input. `pr4xis-chat`'s
    /// `extract_entity_name` has a matching guard on this word, treating it
    /// as invisible to the answer layer, never a real surface.
    pub fn unresolved() -> Self {
        Sem::Pred {
            word: "?".into(),
            role: GrammaticalRole::Argument,
            provenance: PredProvenance::Lexical,
        }
    }

    /// Is this a question?
    pub fn is_question(&self) -> bool {
        matches!(self, Sem::Question { .. })
    }

    /// Is this a proposition?
    pub fn is_proposition(&self) -> bool {
        matches!(self, Sem::Prop { .. })
    }

    /// The queried-entity name this constituent contributes, or `None` if it is
    /// syncategorematic (a functor — the copula / wh-word). Steedman (2000): only
    /// argument-role constituents are the queried entities of a copular question;
    /// a functor contributes a function, never an NP. The WordNet-independent gate
    /// that drops the copula surface — even the verb-homonym `is` — structurally.
    ///
    /// Recurses THROUGH a [`Func`](Sem::Func) into its absorbed arguments,
    /// because a partially-applied constituent (`"is a dog"` reducing to
    /// `S[q]/NP`) wraps its real argument (`dog`) inside a `Func` whose head
    /// is the copula: the argument is in `body`, not the functor head. A
    /// `Func` with no absorbed arguments yet (`body` empty — the fresh-lex
    /// state) yields `None`, so the copula never counts as a queried entity
    /// — the leak this removes.
    pub fn argument_name(&self) -> Option<String> {
        match self.argument_leaf() {
            Some(Sem::Concept { word, .. } | Sem::Pred { word, .. }) => Some(word.clone()),
            // `argument_leaf` yields only Concept/Pred leaves; anything else
            // contributes no queried-entity name.
            _ => None,
        }
    }

    /// The queried-entity LEAF this constituent contributes — the same
    /// resolution as [`argument_name`](Self::argument_name), returning the leaf
    /// `Sem` itself so callers can read its typed provenance (a
    /// [`PredProvenance::Composite`] definiendum routes differently in the
    /// answer layer than a lexical one).
    pub fn argument_leaf(&self) -> Option<&Sem> {
        match self {
            Sem::Concept {
                role: GrammaticalRole::Argument,
                ..
            }
            | Sem::Pred {
                role: GrammaticalRole::Argument,
                ..
            } => Some(self),
            // A functor-role atom (the copula surface, a wh-word) is not a queried
            // entity.
            Sem::Concept {
                role: GrammaticalRole::Functor,
                ..
            }
            | Sem::Pred {
                role: GrammaticalRole::Functor,
                ..
            } => None,
            // Recurse into a partial application to reach the wrapped argument
            // — the first absorbed argument that is itself a real argument
            // leaf (an unabsorbed functor placeholder among the absorbed
            // arguments, e.g. a bare verb argument, recurses to `None` and is
            // skipped).
            Sem::Func { body, .. } => body.iter().find_map(Sem::argument_leaf),
            // A whole clause is never an argument leaf.
            Sem::Prop { .. } | Sem::Question { .. } => None,
        }
    }

    /// The composite halves of this constituent's argument leaf, or `None` for
    /// a lexical leaf (or no leaf) — the typed accessor the answer layer's
    /// definiendum guard reads; never re-parsed from the space in the word.
    pub fn argument_composite_parts(&self) -> Option<(&str, &str)> {
        match self.argument_leaf() {
            Some(Sem::Pred {
                provenance: PredProvenance::Composite { func, arg },
                ..
            }) => Some((func.as_str(), arg.as_str())),
            _ => None,
        }
    }

    /// The comparison-relation kind + EVERY named participant this
    /// constituent's body carries — an ADDITIVE sibling of
    /// [`argument_leaf`](Self::argument_leaf), which deliberately returns
    /// only the FIRST argument leaf (a contract every other caller —
    /// `argument_name`, `argument_composite_parts`, `pr4xis_chat`'s
    /// `entity_leaves` — depends on and which this method leaves
    /// untouched). A "difference between X and Y" question (Barker 2011
    /// derived relational noun, see `comparison_relation_lexicon`)
    /// resolves to a [`Sem::Func`] whose `word` is a LOADED comparison-
    /// relation head and whose `body` holds every named participant — and
    /// relying on `argument_leaf`'s single-leaf contract for this shape
    /// would non-deterministically recite only whichever participant
    /// happens to be listed FIRST, dropping the other(s) silently.
    ///
    /// Recurses into every absorbed argument (mirroring `argument_leaf`'s
    /// own recursion through a partial application) so a comparison nested
    /// under an outer functor (the copula "is", the wh-word "what") is
    /// still found. Requires at least two resolved participant NAMES
    /// (`argument_name` per body element) before returning `Some` — a
    /// registered comparison head with fewer than two overt participants
    /// is not the construction this method exists for, and falls through
    /// to recursing into the body instead of returning a degenerate
    /// single-participant "comparison".
    pub fn comparison_leaves(&self, en: &dyn LexicalReasoner) -> Option<(ConceptRef, Vec<String>)> {
        match self {
            Sem::Func { word, body } => {
                if let Some(kind) = en.comparison_relation_for_surface(word) {
                    let names: Vec<String> = body.iter().filter_map(Sem::argument_name).collect();
                    if names.len() >= 2 {
                        return Some((kind, names));
                    }
                }
                body.iter().find_map(|s| s.comparison_leaves(en))
            }
            _ => None,
        }
    }
}

/// Assign a lexical semantic value to a word based on its Lambek type and on
/// whether that word is USED or MENTIONED ([`ExpressionUse`]).
/// This is the LEXICAL part of the functor — mapping words to their semantic domains.
fn lex(token: &TypedToken, en: &dyn LexicalReasoner) -> Sem {
    let TypedToken {
        word,
        lambek_type: ty,
        expression_use,
    } = token;
    // A MENTIONED expression denotes ITSELF, not what it ordinarily names
    // (Quine 1940 §4 — [`ExpressionUse`]'s own doc carries the citation), so
    // its semantic value is settled BEFORE the type dispatch below and does
    // not vary with which category the chart chose for it. The tokenizer
    // offers a quote-collapsed mention exactly two readings — a saturated
    // `NP` and the close-apposition `NP\NP` of "the term “X”"
    // (`tokenize`'s own `forces_np` branch) — and under BOTH the mention
    // still denotes the expression; only how it COMBINES differs, which is
    // the Lambek type's job, not the semantic value's.
    //
    // Deliberately NO `en.lookup` here: a mention's WordNet senses are the
    // referents of the USED word (see [`Sem::Concept`]'s `expression_use`
    // doc). `role` still comes from the type, so a mention taking the
    // `NP\NP` reading stays a Functor exactly as its `Sem::Func` did before
    // this field existed — `apply`'s close-apposition branch is what
    // promotes it to the sentence's subject.
    if *expression_use == ExpressionUse::Mentioned {
        return Sem::Concept {
            word: word.clone(),
            concepts: Vec::new(),
            role: GrammaticalRole::from_lambek_type(ty),
            expression_use: ExpressionUse::Mentioned,
        };
    }
    let word: &str = word;
    let concepts: Vec<ConceptId> = en.lookup(word).to_vec();
    // The parse-time grammatical role rides on the semantic value (Steedman 2000):
    // an atom-typed token is an Argument, a slash-typed one a Functor. Populated
    // here from the token's Lambek type — the datum interpretation previously
    // discarded.
    let role = GrammaticalRole::from_lambek_type(ty);

    match ty {
        // NP → Entity domain
        LambekType::Atom(super::types::AtomicType::NP) => Sem::Concept {
            word: word.into(),
            concepts,
            role,
            expression_use: ExpressionUse::Used,
        },
        // N → Predicate domain
        LambekType::Atom(super::types::AtomicType::N) => Sem::Pred {
            word: word.into(),
            role,
            provenance: PredProvenance::Lexical,
        },
        // A predicate adjective (`S[adj]\NP`) is slash-typed yet fills the
        // copula's argument slot: it is the θ-marked complement — the property the
        // clause predicates — NOT a syncategorematic functor (Montague; Steedman
        // 2000 predicate complements; Bowers 1993 predication). It surfaces as an
        // Argument-role `Pred` so `argument_name` reaches it as the definiendum of
        // "what is {adj}". Without this it lex'd to a `Func{Functor}` and the
        // adjective was dropped, so "what is able" extracted zero entities and
        // could not be defined.
        _ if *ty == super::types::svo::predicate_adjective() => Sem::Pred {
            word: word.into(),
            role: GrammaticalRole::Argument,
            provenance: PredProvenance::Lexical,
        },
        // A/B or A\B → Function domain
        // The function takes a B-domain value and produces an A-domain value.
        // No argument has been absorbed yet — `body` starts empty.
        LambekType::RightDiv(_, _) | LambekType::LeftDiv(_, _) => Sem::Func {
            word: word.into(),
            body: Vec::new(),
        },
        // S or other atoms — predicate as default
        _ => Sem::Pred {
            word: word.into(),
            role,
            provenance: PredProvenance::Lexical,
        },
    }
}

/// Apply the functor: compose semantic values over the SAME derivation space
/// the syntax chart searches, not a separate simplified re-derivation.
///
/// A single greedy left-to-right adjacent-pair pass (the previous
/// implementation) commits to whichever reduction is available FIRST, which
/// need not be the bracketing that leads to a complete sentence — e.g. "is
/// Medicaid's contractor a company" greedily reduces "is"+"Medicaid" (both
/// immediately adjacent and NP-typed) before "Medicaid's contractor" ever
/// gets a chance to compose, stranding "is" partially applied with the wrong
/// argument while "'s", "contractor", and "a company" are left as unreduced
/// leftovers the old code silently discarded (`values.into_iter().next()`).
/// The syntax chart ([`chart_reduce`](super::reduce::chart_reduce)) does not
/// have this problem because it is a CYK chart exploring every split, not a
/// greedy linear scan — so a sentence could be `parsed = true` (chart
/// succeeds) while `interpret` returned a stuck, wrong `Sem::Func` for a
/// completely different, failed bracketing: a real defect, not merely
/// cosmetic, since the answer layer (`chat::attempt_partial_understanding`'s
/// `KnownKnown` branch) had to consume whatever `interpret` produced.
///
/// This is exactly Montague's own compositionality thesis — syntax and
/// semantics are computed by the SAME derivation, not two independent walks
/// that happen to usually agree (Montague 1970, "Universal Grammar",
/// *Theoria* 36(3); Steedman 2000, *The Syntactic Process*, MIT Press, the
/// CCG "rule-to-rule" hypothesis: every syntactic combinator has a
/// corresponding semantic composition rule, applied together, never
/// separately re-derived; Blackburn & Bos 2005, *Representation and
/// Inference for Natural Language*, Ch. 2-3, the standard CYK chart with a
/// semantic value attached to every cell). This function now IS that: a CYK
/// chart over `tokens`' (already singly-typed, e.g. from the syntax chart's
/// own winning-derivation backtrack — `extract_winning_types` in
/// `super::reduce`) types, building `Sem` at every cell via the `apply`
/// composition rule below exactly where the syntax chart would reduce a
/// type — so semantics finds a complete derivation whenever ANY split does,
/// not only the first adjacent pair.
pub fn interpret(tokens: &[TypedToken], en: &dyn LexicalReasoner) -> Sem {
    interpret_with_unary_rules(tokens, en, &[])
}

/// The semantic side of a LOADED unary (type-changing) rule: for a cell
/// already deriving `from`, ALSO derive `to`, built by applying `build` to
/// the `from`-typed `Sem`. Mirrors [`super::reduce::close_unary`]'s own
/// fixpoint closure exactly, minus the cost bookkeeping `interpret`'s
/// simpler "first-found wins" chart never needed (see `interpret`'s own doc
/// comment).
fn close_unary_semantic(
    cell: &mut hashbrown::HashMap<LambekType, Sem>,
    unary_rules: &[(LambekType, LambekType)],
) {
    if unary_rules.is_empty() {
        return;
    }
    loop {
        let mut additions: Vec<(LambekType, Sem)> = Vec::new();
        for (t, sem) in cell.iter() {
            for (from, to) in unary_rules {
                if from == t && !cell.contains_key(to) {
                    additions.push((to.clone(), promote_type_change(from, to, sem)));
                }
            }
        }
        if additions.is_empty() {
            break;
        }
        let mut changed = false;
        for (t, s) in additions {
            if let hashbrown::hash_map::Entry::Vacant(v) = cell.entry(t) {
                v.insert(s);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

/// The N → NP domain conversion a determiner-headed NP ("the" + "dog" → NP,
/// the NP-result branch of [`apply`]) and the determiner-LESS unary
/// promotion rule ([`promote_bare_np`]) both need: a LEXICAL N-typed `Pred`
/// (single loaded word) becomes a plain `Sem::Concept` — unchanged from this
/// codebase's original behavior. A COMPOSITE `Pred` (an N/N-modifier chain's
/// minted concatenation, `PredProvenance::Composite` — e.g. "six", "required",
/// and "elements" reducing to the composite "six required elements") is left
/// AS A `Pred`, provenance intact, rather than converted: `Sem::Concept`
/// carries no provenance field at all (see
/// [`crate::social::judicial::statute_structure::grounding`]'s own doc for
/// why that absence is a DELIBERATE, load-bearing property elsewhere), so
/// converting a composite here would silently erase the func/arg breakdown
/// axiom `DefiniendumIsALexicalUnit` (pr4xis-chat) depends on to report only
/// the failing LEXICAL LEAVES of an unresolvable N/N compound. Confirmed
/// real regression this fixes: "What are the six required elements for
/// EVV?" — the determiner "the" applying to the composite N "six required
/// elements" discarded its Composite provenance, so the answer layer saw a
/// bare `Sem::Concept{word: "six required elements"}` with no leaves to
/// decompose and reported the entire fabricated 3-word concatenation as one
/// unknown "word", rather than the genuinely non-lexical leaf
/// (`unresolved_surfaces`'s own `entity_composite_parts` guard, designed for
/// exactly this shape, could not see it once it had already become a
/// `Concept`). `argument_leaf`/`extract_word`/`extract_predicate` already
/// treat an Argument-role `Pred` and `Concept` identically, so leaving a
/// composite as `Pred` here changes no downstream domain semantics — only
/// which leaf-decomposition path the answer layer can use.
fn n_to_np_argument(sem: &Sem) -> Sem {
    match sem {
        Sem::Pred {
            provenance: composite @ PredProvenance::Composite { .. },
            word,
            role,
        } => Sem::Pred {
            word: word.clone(),
            role: *role,
            provenance: composite.clone(),
        },
        // An `N → NP` promotion of a common noun: ordinary running prose, so
        // the promoted entity is USED. A MENTIONED token never reaches here —
        // `lex` never gives it an `N` reading at all (`tokenize`'s `forces_np`
        // branch withholds the common-noun category from a quoted span).
        Sem::Pred { word, .. } => Sem::Concept {
            word: word.clone(),
            concepts: Vec::new(),
            role: GrammaticalRole::Argument,
            expression_use: ExpressionUse::Used,
        },
        other => other.clone(),
    }
}

/// The N → NP promotion's semantics — see [`n_to_np_argument`] for the
/// shared conversion rule.
fn promote_bare_np(sem: &Sem) -> Sem {
    n_to_np_argument(sem)
}

/// The semantics of a CLAUSAL-ADJUNCT type change — CCGbank's §3.8 schema
/// `S$ ⇒ X|X` (Hockenmaier & Steedman 2005, *CCGbank User's Manual*,
/// MS-CIS-05-09, p. 55, (53); instantiated at (54)a as
/// `S[pss]\NP ⇒ NP\NP`, "workers [exposed to it]"): a saturated-but-for-its-
/// subject VERB PHRASE becomes a MODIFIER of the constituent it attaches to.
///
/// The distinction that matters downstream is DERIVED vs FRESH. `apply`'s
/// `NP\NP` branch tells a close-apposition LEAF (a quoted definiendum that
/// takes over reference from its head — "the term 'X'") apart from a DERIVED
/// `NP\NP` modifier (a PP such as "of Commerce", which leaves the head's
/// reference intact) by exactly one signal: `body.is_empty()` — see that
/// branch's own comment for the full rationale. A type-changed VP is a
/// DERIVED modifier in precisely that sense: §3.8's whole point is that the
/// constituent already "obtains" its own category as a head node before the
/// rule turns it into an adjunct. So its `Sem::Func` is marked derived by
/// carrying its own predicate in `body`, and the head NP's reference
/// survives the application — the same outcome a PP already gets.
fn promote_clausal_adjunct(sem: &Sem) -> Sem {
    match sem {
        Sem::Func { word, body } if body.is_empty() => Sem::Func {
            word: word.clone(),
            body: alloc::vec![Sem::Pred {
                word: word.clone(),
                role: GrammaticalRole::Functor,
                provenance: PredProvenance::Lexical,
            }],
        },
        other => other.clone(),
    }
}

/// Dispatch a loaded unary rule to its own semantics, by the rule's OWN
/// category shape — the same type-shaped dispatch `apply` uses throughout
/// this module, never a surface or a flag.
///
/// A rule OUT of an atomic category into an atomic category is a promotion
/// of content (`N → NP`, [`promote_bare_np`]); a rule out of a SLASH
/// category (a verb phrase) into a modifier is CCGbank's clausal-adjunct
/// type change ([`promote_clausal_adjunct`]).
fn promote_type_change(from: &LambekType, to: &LambekType, sem: &Sem) -> Sem {
    match (from, to) {
        (LambekType::LeftDiv(_, _) | LambekType::RightDiv(_, _), LambekType::LeftDiv(_, _)) => {
            promote_clausal_adjunct(sem)
        }
        _ => promote_bare_np(sem),
    }
}

/// [`interpret`], additionally closing every chart cell (leaf AND derived
/// span alike) under `unary_rules` — `(from, to)` type-changing pairs applied
/// via `close_unary_semantic`/`promote_bare_np` (private helpers, just
/// below). Every existing call site passes `&[]` through [`interpret`] and
/// is BYTE-IDENTICAL to before this function existed (`close_unary_semantic`
/// is a guaranteed no-op on an empty slice). The one caller that supplies a
/// non-empty slice is
/// [`crate::social::judicial::statute_structure::grounding::defines_pointers`]
/// — see [`super::supertag_costs::SupertagCostTable::with_extra_unary`]'s own
/// doc for why a bare-noun-phrase promotion is licensed there and NOT in the
/// shared grammar every other caller of [`interpret`] uses.
pub fn interpret_with_unary_rules(
    tokens: &[TypedToken],
    en: &dyn LexicalReasoner,
    unary_rules: &[(LambekType, LambekType)],
) -> Sem {
    let n = tokens.len();
    let Some(chart) = build_semantic_chart(tokens, en, unary_rules) else {
        return Sem::Pred {
            word: "empty".into(),
            role: GrammaticalRole::Argument,
            provenance: PredProvenance::Lexical,
        };
    };
    match select_cell_goal(&chart[0][n]) {
        Some(sem) => sem.clone(),
        None => Sem::Pred {
            word: "?".into(),
            role: GrammaticalRole::Argument,
            provenance: PredProvenance::Lexical,
        },
    }
}

/// Every MAXIMAL, non-overlapping sub-span of `tokens` whose reading `accept`s
/// — the semantic half of the PARTIAL-PARSE goal
/// ([`super::reduce::clause_fragments_with_costs_bounded`] is the syntactic
/// half), for a caller looking for a SPECIFIC predication inside running text
/// rather than asking whether the whole string is one sentence.
///
/// The chart this scans is the SAME one [`interpret_with_unary_rules`] builds
/// and then reads a single cell of. A CYK chart is a well-formed substring
/// table (Sheil, "Observations on Context-Free Parsing", 1976): every
/// derivable sub-span's meaning is already composed and sitting in
/// `chart[i][j]` — the whole-string goal simply discards all of it. So this
/// costs one chart, exactly as [`interpret_with_unary_rules`] does, and
/// clones only the readings `accept` takes.
///
/// Why sub-spans matter even when the whole string DOES derive: an adjunct
/// that composes at the top (`S + S\S → S` — a trailing infinitival purpose
/// clause, a sentence-final adverbial) leaves the whole-string cell holding
/// the ADJUNCT's meaning, not the predication's, so a caller matching on
/// predication shape finds nothing at `[0, n)` while the predication itself
/// sits complete one cell down. Abney's constituency/attachment separation
/// ("Parsing By Chunks", in Berwick, Abney & Tenny (eds.),
/// *Principle-Based Parsing*, Kluwer 1991, §1–§3) is the standing argument
/// that these are two different questions and that answering the second badly
/// must not destroy the first.
///
/// Deterministic: spans are visited LONGEST first (Abney, "Partial Parsing via
/// Finite-State Cascades", *Natural Language Engineering* 2(4), 1996, §1 —
/// longest-match), ties leftmost; within a cell, readings are visited in
/// `select_cell_goal`'s own preference order, so the reading accepted here
/// is the one the whole-string goal would have reported for that span. A span
/// overlapping an already-accepted one is skipped, so the result is a set of
/// disjoint maximal readings in input order.
pub fn interpret_maximal_spans_where(
    tokens: &[TypedToken],
    en: &dyn LexicalReasoner,
    unary_rules: &[(LambekType, LambekType)],
    accept: &mut dyn FnMut(&Sem) -> bool,
) -> Vec<(TokenSpan, Sem)> {
    let n = tokens.len();
    let Some(chart) = build_semantic_chart(tokens, en, unary_rules) else {
        return Vec::new();
    };
    let mut taken = vec![false; n];
    let mut out: Vec<(TokenSpan, Sem)> = Vec::new();
    for width in (1..=n).rev() {
        for i in 0..=(n - width) {
            let j = i + width;
            if taken[i..j].iter().any(|t| *t) {
                continue;
            }
            let Some(sem) = ordered_cell_readings(&chart[i][j])
                .into_iter()
                .find(|sem| accept(sem))
            else {
                continue;
            };
            for slot in &mut taken[i..j] {
                *slot = true;
            }
            out.push((TokenSpan::new(i, j), sem.clone()));
        }
    }
    out.sort_by_key(|(span, _)| *span);
    out
}

/// A cell's readings in [`select_cell_goal`]'s preference order (S-family
/// first, interrogative-featured ahead of other featured ahead of bare, then
/// the types' own total order) — the shared ordering both the whole-string
/// goal and [`interpret_maximal_spans_where`] read a cell through, so neither
/// can prefer a reading the other would have rejected.
fn ordered_cell_readings(cell: &hashbrown::HashMap<LambekType, Sem>) -> Vec<&Sem> {
    let mut readings: Vec<(&LambekType, &Sem)> = cell.iter().collect();
    readings.sort_by_key(|(t, _)| cell_goal_key(t));
    readings.into_iter().map(|(_, sem)| sem).collect()
}

/// The goal preference key — factored out of [`interpret_with_unary_rules`]'s
/// own `min_by_key` UNCHANGED, so the whole-string goal and every sub-span
/// goal apply the identical criterion.
fn cell_goal_key(t: &LambekType) -> (u8, LambekType) {
    let tier: u8 = match t {
        LambekType::Atom(super::types::AtomicType::S(Some(
            super::types::SentenceFeature::Q | super::types::SentenceFeature::Wq,
        ))) => 0,
        LambekType::Atom(super::types::AtomicType::S(Some(_))) => 1,
        LambekType::Atom(super::types::AtomicType::S(None)) => 2,
        _ => 3,
    };
    (tier, t.clone())
}

/// Goal: an S-family type at the span wins first, preferring
/// interrogative-featured (S[q]/S[wq]) over a bare/other-featured S — the
/// SAME tier `chart_reduce`'s own goal selection uses (Hockenmaier 2003
/// §5.12), so semantics and syntax never disagree about which reading of
/// an ambiguous parse won. [`interpret`] is also called directly
/// on SUB-sentential fragments by callers/tests that want whatever type
/// the fragment naturally reduces to (a bare NP for "the term
/// 'emolument'", not a whole clause) — so when no S-family type derived,
/// fall back to any derivation at all (deterministic: `LambekType`
/// has a total order), and only report nothing when the cell is completely
/// empty.
fn select_cell_goal(cell: &hashbrown::HashMap<LambekType, Sem>) -> Option<&Sem> {
    cell.iter()
        .min_by_key(|(t, _)| cell_goal_key(t))
        .map(|(_, sem)| sem)
}

/// Build the semantic CYK chart — [`interpret_with_unary_rules`]'s own body,
/// factored out UNCHANGED so a sub-span goal
/// ([`interpret_maximal_spans_where`]) can read the SAME completed chart
/// rather than re-composing. `None` when the input is empty or exceeds the
/// interpreter's width bound.
fn build_semantic_chart(
    tokens: &[TypedToken],
    en: &dyn LexicalReasoner,
    unary_rules: &[(LambekType, LambekType)],
) -> Option<Vec<Vec<hashbrown::HashMap<LambekType, Sem>>>> {
    // O(n³) in the token count (one type per token here, so no alternatives
    // dimension — this chart is cheaper than the syntax chart's O(n³K²)), so
    // a pathologically long utterance is still a resource-exhaustion DoS.
    // Real sentences are far under this bound (matching chart_reduce's
    // MAX_CHART_WIDTH); abstain past it.
    const MAX_INTERPRET_WIDTH: usize = 256;
    let n = tokens.len();
    if n == 0 || n > MAX_INTERPRET_WIDTH {
        return None;
    }

    // chart[i][j]: every LambekType derivable for tokens[i..j], each paired
    // with the Sem its (first-found, deterministic) derivation composed.
    // With one type per leaf, a span can still derive more than one result
    // type via different splits (e.g. an N-result vs an NP-result reading at
    // the same span), so the cell is a map, not a single value.
    let mut chart: Vec<Vec<hashbrown::HashMap<LambekType, Sem>>> = (0..=n)
        .map(|_| (0..=n).map(|_| hashbrown::HashMap::new()).collect())
        .collect();

    for (i, tok) in tokens.iter().enumerate() {
        chart[i][i + 1].insert(tok.lambek_type.clone(), lex(tok, en));
        close_unary_semantic(&mut chart[i][i + 1], unary_rules);
    }

    for span in 2..=n {
        for i in 0..=(n - span) {
            let j = i + span;
            for k in (i + 1)..j {
                let left: Vec<(LambekType, Sem)> = chart[i][k]
                    .iter()
                    .map(|(t, s)| (t.clone(), s.clone()))
                    .collect();
                let right: Vec<(LambekType, Sem)> = chart[k][j]
                    .iter()
                    .map(|(t, s)| (t.clone(), s.clone()))
                    .collect();
                for (t_left, s_left) in &left {
                    for (t_right, s_right) in &right {
                        if let Some(t_result) = super::types::reduce(t_left, t_right) {
                            let is_forward = matches!(t_left, LambekType::RightDiv(_, _));
                            let sem = if is_forward {
                                apply(s_left, s_right, t_left, &t_result, en)
                            } else {
                                apply(s_right, s_left, t_right, &t_result, en)
                            };
                            // First-found wins (deterministic: (i,k) iterate in
                            // a fixed order) — matching `chart_reduce`'s own
                            // "the chart keeps whichever reading derives a
                            // complete parse" idiom, just without a cost table
                            // to arbitrate multiple derivations of the SAME
                            // type (montague never needed one: the syntax
                            // chart already picked the winning type per span).
                            chart[i][j].entry(t_result).or_insert(sem);
                        }
                    }
                }
            }
            close_unary_semantic(&mut chart[i][j], unary_rules);
        }
    }

    Some(chart)
}

/// Semantic function application — the ONLY composition rule.
/// When types reduce via A/B + B → A, the semantics is f(x).
/// The result domain is determined by the result type. `func_type` is the
/// FUNCTOR's own Lambek type (as opposed to `result_type`, the type the
/// reduction produces) — needed to distinguish composition shapes that
/// share the same atomic result (e.g. determiner+noun NP/N + N → NP versus
/// close-apposition NP + NP\NP → NP both yield an NP result, but mean
/// different things: see the NP-result branch below).
fn apply(
    func: &Sem,
    arg: &Sem,
    func_type: &LambekType,
    result_type: &LambekType,
    en: &dyn LexicalReasoner,
) -> Sem {
    match result_type {
        // Result is S (any feature) — check if question or proposition
        LambekType::Atom(super::types::AtomicType::S(feature)) => {
            // A FRONTED SCOPE-SETTING SENTENTIAL ADJUNCT ("For purposes of
            // this subsection," / "In this subsection," / "Except for the
            // purposes of X," / "Subject to Y,") is SYNCATEGOREMATIC at the
            // clause level — Steedman (2000) *The Syntactic Process*, Ch. 2's
            // adjunct/argument distinction, the same distinction
            // `GrammaticalRole::from_lambek_type` already applies at the NP
            // level to the copula: an `S/S` adjunct scopes WHERE/WHEN/UNDER
            // WHAT CONDITION a proposition holds, but contributes no argument
            // of its own to that proposition. So once the adjunct has
            // absorbed its own complement (down to the saturated `S/S`
            // shape — see `svo::fronted_scope_adjunct_np`/`_pp`'s own docs
            // for how both variants get there) and meets the REST clause on
            // its right, the REST's own meaning passes through UNCHANGED —
            // the clause-level mirror of the NP-result branch's
            // close-apposition guard below, which drops the syncategorematic
            // functor and promotes the real content instead. Gated on BOTH
            // the derived TYPE shape (`S/S` exactly) and the functor's own
            // SURFACE (`tokenize::is_fronted_scope_adjunct_head`) — the
            // double guard `svo::nominal_coordinator_np`'s own doc comment
            // establishes as this codebase's standing discipline for a
            // semantically load-bearing dispatch, even where (as here) the
            // shape alone is not currently known to collide with anything
            // else in the type system.
            if let LambekType::RightDiv(a, b) = func_type
                && matches!(**a, LambekType::Atom(super::types::AtomicType::S(_)))
                && matches!(**b, LambekType::Atom(super::types::AtomicType::S(_)))
                && super::tokenize::is_fronted_scope_adjunct_head(&extract_predicate(func))
            {
                return arg.clone();
            }
            // A SENTENTIAL WH-QUESTION COORDINATOR (`svo::
            // sentential_coordinator_wq`, `(S[wq]\S[wq])/S[wq]` — "What Is
            // Medicaid, and Who Is Eligible?") reaches this SECOND
            // application (having already absorbed its right conjunct down
            // to the derived `S[wq]\S[wq]` shape on its first) with the
            // LEFT conjunct as `arg`. Per this task's own scoping design
            // (mirroring the corpus's own `classify_case`, which only
            // requires the LEFT conjunct's answer to be present, and the
            // fact that the right conjunct's subject is almost always the
            // anaphoric pronoun "it" — resolving it would need
            // coreference/anaphora machinery, Kamp 1981 DRT; Grosz, Joshi &
            // Weinstein 1995 Centering Theory, this codebase does not have):
            // keep only the LEFT conjunct's own `Sem::Question`, dropping
            // the coordinator and the right conjunct — the SAME
            // "syncategorematic functor, promote the real content instead"
            // move the fronted-scope-adjunct branch just above makes,
            // mirrored to the OTHER side (there the adjunct is the functor
            // on the LEFT absorbing the REST on its right; here the
            // coordinator+right-conjunct composite is the functor on the
            // RIGHT absorbing the LEFT conjunct as `arg`). Gated on BOTH the
            // derived TYPE shape (bare `S[wq]\S[wq])` exactly — DISTINCT
            // from the fronted-adjunct's `S/S` shape above, opposite
            // division direction, so no collision) and the functor's own
            // SURFACE (`tokenize::nominal_coordinator_canonical`) — the SAME
            // double-guard discipline `svo::nominal_coordinator_np`'s own
            // doc establishes.
            if let LambekType::LeftDiv(a, b) = func_type
                && matches!(**a, LambekType::Atom(super::types::AtomicType::S(_)))
                && matches!(**b, LambekType::Atom(super::types::AtomicType::S(_)))
                && super::tokenize::nominal_coordinator_canonical(&extract_predicate(func))
                    .is_some()
            {
                return arg.clone();
            }
            // When the argument is a RELATIONAL predicative complement — a `Func`
            // whose head surface is a LOADED relation ("part of") — the asserted
            // relation comes from the COMPLEMENT, not the copula: lift its surface
            // to the predicate and flatten its object into the arguments. So
            // "is X part of Y" → Question{ "part of", [X, Y] }, whereas the plain
            // copula "is X a Y" keeps the function's predicate ("is" → the
            // Subsumption default). The discriminator is loaded data
            // (`relation_for_surface`), not a hardcoded "part of" match.
            let (predicate, arguments) = match arg {
                Sem::Func { word, body } if en.relation_for_surface(word).is_some() => {
                    let mut arguments = extract_arguments(func);
                    arguments.extend(body.iter().cloned());
                    (word.clone(), arguments)
                }
                _ => {
                    let mut arguments = extract_arguments(func);
                    arguments.push(arg.clone());
                    (extract_predicate(func), arguments)
                }
            };
            match feature {
                Some(super::types::SentenceFeature::Q) => Sem::Question {
                    predicate,
                    arguments,
                    illocution: QuestionIllocution::Polar,
                },
                Some(super::types::SentenceFeature::Wq) => Sem::Question {
                    predicate,
                    arguments,
                    illocution: QuestionIllocution::Content,
                },
                _ => Sem::Prop {
                    predicate,
                    arguments,
                },
            }
        }
        // Result is NP (entity)
        LambekType::Atom(super::types::AtomicType::NP) => {
            // A MEDIAL SUBJECT-VERB SUPPLEMENT (defines-lens gap G2 — "the
            // term 'X', used with respect to Y, means ..." / "the term 'X',
            // as used in this title, means ...") is syncategorematic at the
            // NP level — Huddleston & Pullum (2002) Ch. 15 "Supplements"
            // §1: unlike a close-apposition NP (which RESTATES the head
            // NP's referent and takes over reference, the branch just
            // below), a supplement merely SCOPES the head NP and
            // contributes nothing of its own. So the SUBJECT's own concept
            // (`arg` — the functor here, `func`, sits on the RIGHT of a
            // backward application: SUBJECT + adjunct:NP\NP → NP) survives
            // unchanged — the exact clause-level move the fronted
            // scope-adjunct S-result branch above makes for `S/S`, just at
            // the NP level. `medial_supplement_np` is STRUCTURALLY
            // IDENTICAL to `close_apposition` (both `NP\NP`; see
            // `svo::medial_supplement_np`'s own doc for why the collision
            // is genuine and expected) — checked FIRST, gated on the SAME
            // double guard (type shape AND the reserved synthetic marker
            // `tokenize::is_medial_supplement_marker`, never a real lexical
            // word) `nominal_coordinator_np`'s doc establishes as this
            // codebase's standing discipline, so a genuine quoted
            // close-apposition leaf (whose word is never this marker) still
            // falls through to the close-apposition branch below unaffected.
            if *func_type == super::types::svo::medial_supplement_np()
                && let Sem::Func { word, body } = func
                && body.is_empty()
                && super::tokenize::is_medial_supplement_marker(word)
            {
                return arg.clone();
            }
            // Close apposition (NP\NP): the quoted definiendum's own
            // concept takes over reference — "the term" (the head NP)
            // contributes nothing further, exactly as "the" contributes
            // nothing further to "the term" below. Guarded on the FUNCTOR's
            // own type — BUT the bare NP\NP shape is NOT unique to a quoted
            // apposition leaf: `svo::preposition` is `(NP\NP)/NP`, so a PP
            // ("of Commerce") reduces to that SAME NP\NP type once it has
            // absorbed its own object, and would then reach this branch as
            // `func` too ("the Secretary" + "of Commerce" → NP). The second
            // guard — `body.is_empty()` — is what tells them apart: a
            // close-apposition LEAF is a fresh, never-yet-applied `Func`
            // (lexed straight from the quoted token, absorbing nothing
            // before it meets its head NP), whereas a preposition's NP\NP is
            // always DERIVED — it only reaches this shape by first absorbing
            // its object, so its `body` already carries that argument. A
            // close-apposition category never itself absorbs an argument
            // (NP\NP is fully saturated after one application), so this is
            // the only moment it can appear here with an empty body.
            // NOMINAL COORDINATION (defines-lens gap G4(a) — "an unpaid
            // family member, a foster parent, or another unpaid
            // individual"): the SAME bare `NP\NP` shape ONE MORE
            // construction reaches here — `svo::nominal_coordinator_np`
            // (`(NP\NP)/NP`), after absorbing its own right NP conjunct, is
            // structurally IDENTICAL to `close_apposition`/
            // `medial_supplement_np` (all three are plain `NP\NP` — see
            // `svo::nominal_coordinator_np`'s own doc for why this
            // collision is genuine and expected). Disambiguated the SAME
            // way as the two branches above: by the functor's own SURFACE,
            // never the type shape alone — `tokenize::
            // nominal_coordinator_canonical` recognizes the closed set of
            // real coordinator words ("and"/"or") AND
            // `tokenize::find_list_coordinator_commas`'s synthetic list-
            // coordinator markers, so an ordinary quoted close-apposition
            // leaf (whose word is never one of these) still falls through
            // to the close-apposition branch below unaffected.
            if *func_type == super::types::svo::close_apposition()
                && let Sem::Func { word, body } = func
                && let Some(canonical) = super::tokenize::nominal_coordinator_canonical(word)
            {
                return flatten_coordination(canonical, arg, body);
            }
            // COORDINATED CLOSE-APPOSITION DEFINIENDA (defines-lens gap G5
            // — "the terms 'exploitation' and 'financial exploitation'
            // mean ..."): the coordinator has ALREADY absorbed every
            // conjunct on its right via the generic accumulate branch
            // below (`RightDiv`/`LeftDiv` result branch) — each one a raw,
            // never-yet-applied quoted leaf (`body.is_empty()` per
            // conjunct, the SAME "fresh Func" signature the close-
            // apposition single-leaf check just below relies on) — and
            // THIS is the final application that would otherwise drop the
            // whole coordinated set against the head NP ("the terms") the
            // way a lone close-apposition leaf drops "the term" below.
            // Recognized by the RESERVED marker
            // `tokenize::apposition_coordinator_canonical` — never the
            // literal "and"/"or" surface `nominal_coordinator_canonical`
            // already claims for the check just above, which reaches this
            // SAME `NP\NP` shape for the STRUCTURALLY DIFFERENT
            // definiens-side (object) coordination
            // (`svo::nominal_coordinator_apposition`'s own doc has the
            // full rationale for why a dedicated marker, not surface+type
            // alone, disambiguates the two here). Each conjunct is
            // promoted to its own `Sem::Concept` — mirroring the
            // single-conjunct promotion just below — and the SET survives
            // as one `Sem::Func`, "the terms" (`arg`) dropped exactly as
            // "the term" is for a single conjunct. The result `Func`
            // deliberately keeps the MARKER as its own `word` (never
            // canonicalized to plain "and"/"or") — this is the ONE signal
            // `grounding::definiendum_words` uses to tell a coordinated
            // DEFINIENDUM set apart from an ordinary coordinated NP
            // subject that happens to reduce to the identical `Sem::Func`
            // shape (a REAL, measured regression that function's own doc
            // documents); canonicalizing away the marker here would erase
            // that signal before it ever reaches `defines_pointers`.
            if *func_type == super::types::svo::close_apposition()
                && let Sem::Func { word, body } = func
                && super::tokenize::apposition_coordinator_canonical(word).is_some()
                && !body.is_empty()
            {
                // Each conjunct is already a `Sem::Concept` carrying
                // [`ExpressionUse::Mentioned`] — `lex` settles a quoted
                // mention's semantic value at the leaf, under EITHER of the
                // two categories the tokenizer offers it (see `lex`'s own
                // doc), so a conjunct arrives here as a mention and passes
                // through unchanged. The `Sem::Func` arm remains for a
                // non-mention leaf (a collapsed proper-noun RUN, the OTHER
                // holder of the `NP\NP` close-apposition category), which is
                // promoted USED — `grounding::definiendum_words` then
                // declines it, because a used name is not a definiendum.
                let items: Vec<Sem> = body
                    .iter()
                    .map(|item| match item {
                        Sem::Func { word, body } if body.is_empty() => Sem::Concept {
                            word: word.clone(),
                            concepts: Vec::new(),
                            role: GrammaticalRole::Argument,
                            expression_use: ExpressionUse::Used,
                        },
                        other => other.clone(),
                    })
                    .collect();
                return Sem::Func {
                    word: word.clone(),
                    body: items,
                };
            }
            // CLOSE APPOSITION WITH A MENTIONED HEAD — "the term “X”": the
            // mention takes over reference from "the term", which contributes
            // nothing further. `func` is a `Sem::Concept` rather than the
            // `Sem::Func` the branch below expects because `lex` gives a
            // quoted mention its metalinguistic value at the LEAF, whichever
            // of its two categories the chart picked (that function's own doc
            // has the rationale) — so this is the mention's own promotion,
            // and it is exact: the `Mentioned` marking came from the quote
            // glyphs themselves, not from the `NP\NP` type shape, which
            // `svo::preposition`, `svo::medial_supplement_np`,
            // `svo::nominal_coordinator_np` and a type-changed VP all reach
            // too (see the branches above and `close_apposition`'s own doc).
            //
            // `role` becomes `Argument`: the promoted mention IS the clause's
            // subject now, no longer a postnominal modifier — the same role
            // the pre-mention `Sem::Func` promotion below assigns.
            if *func_type == super::types::svo::close_apposition()
                && let Sem::Concept {
                    word,
                    expression_use: ExpressionUse::Mentioned,
                    ..
                } = func
            {
                return Sem::Concept {
                    word: word.clone(),
                    concepts: Vec::new(),
                    role: GrammaticalRole::Argument,
                    expression_use: ExpressionUse::Mentioned,
                };
            }
            // The SAME promotion for a close-apposition leaf that is NOT a
            // mention — a collapsed proper-noun RUN, the other token class
            // `tokenize`'s `forces_np` branch hands the `NP\NP` category to.
            // Promoted USED: "the United States" names the country, it does
            // not talk about the phrase.
            if *func_type == super::types::svo::close_apposition()
                && let Sem::Func { word, body } = func
                && body.is_empty()
            {
                return Sem::Concept {
                    word: word.clone(),
                    concepts: Vec::new(),
                    role: GrammaticalRole::Argument,
                    expression_use: ExpressionUse::Used,
                };
            }
            // A DERIVED-NOMINAL RELATIONAL-NOUN PP-COMPLEMENT ("the
            // difference between X and Y") — Barker (2011) §1.4 "Derived
            // versus underived relational nominals": a DERIVED relational
            // noun (here, "difference", deverbal from "differ (from)") can
            // overtly express MULTIPLE participants via its own PP
            // complement, unlike an underived relational noun's
            // single-participant ceiling (see
            // `comparison_relation_lexicon`'s module doc for the full
            // citation). `func` here is the PREPOSITION ("between") that
            // has already absorbed its own object (an ordinary
            // `svo::preposition`, the SAME `(NP\NP)/NP` shape "of Commerce"
            // reaches below); `arg` is the HEAD NP ("the difference").
            // Distinguished from the "Secretary of Commerce" PP-MODIFIER
            // case purely by a LOADED lexicon lookup on the HEAD noun
            // (`comparison_relation_for_surface`) — never a
            // preposition-surface literal — so an unregistered head
            // ("secretary") always falls through unchanged below, exactly
            // as before this branch existed (see
            // `a_derived_prepositional_np_np_does_not_trigger_the_apposition_guard`).
            // Placed LAST among the `NP\NP`-specific branches: every guard
            // above is a loaded-surface or type-shape check, never an
            // "else", so this addition cannot reorder or shadow any of
            // them — a genuine close-apposition leaf (`body.is_empty()`,
            // caught above) and a nominal coordination (caught above by
            // `nominal_coordinator_canonical`/`apposition_coordinator_canonical`)
            // both return before reaching here.
            if *func_type == super::types::svo::close_apposition()
                && let Sem::Func {
                    word: _prep_word,
                    body: prep_body,
                } = func
                && let Sem::Concept {
                    word: head_word, ..
                } = arg
                && let Some(_kind) = en.comparison_relation_for_surface(head_word)
            {
                return Sem::Func {
                    word: head_word.clone(),
                    body: flatten_pp_object_conjuncts(prep_body),
                };
            }
            n_to_np_argument(arg)
        }
        // Result is N (predicate) — modifier applied to predicate. The minted
        // surface is a CONCATENATION, not a lexical token, so it carries
        // Composite provenance: if the answer layer cannot resolve it, the
        // failing leaves — not the minted string — are the reportable
        // unknowns (the degenerate `is:N/N + a:N` derivation of "what is a
        // long" minted "is a" here and the chat replied 'I do not know the
        // word "is a"'). A resolving composite ("dormant fault") is untouched.
        LambekType::Atom(super::types::AtomicType::N) => {
            // NOMINAL COORDINATION at the common-noun level (defines-lens
            // gap G4(a) — "bodily injury, impairment, or disease"):
            // `svo::nominal_coordinator_n` (`(N\N)/N`), after absorbing its
            // own right N conjunct, derives `N\N` — so BY THIS POINT
            // (the second application, producing the `N` result) `func_type`
            // is already the intermediate `N\N` shape, not the original
            // undivided `(N\N)/N`. Gated on BOTH the derived TYPE shape
            // (bare `N\N`, matched structurally like the `S[wq]\S[wq]`
            // coordination check above — not `==
            // nominal_coordinator_n()`, which is the shape BEFORE the first
            // application) AND the functor's own SURFACE — the same
            // double-guard discipline `svo::nominal_coordinator_np`'s own
            // doc establishes as this codebase's standing rule for a
            // semantically load-bearing dispatch. Surface alone is not
            // enough: `svo::nominal_modifier_noun` (`N/N`, task #29/#35's
            // bare-nominal-compounding rule) can ALSO be assigned to a
            // token literally spelled "or"/"and" (an arbitrary word never
            // actually typed as a coordinator), and without the type guard
            // this branch would misfire on it, returning a spurious empty
            // coordination instead of the ordinary modifier+noun
            // concatenation below.
            if let LambekType::LeftDiv(a, b) = func_type
                && matches!(**a, LambekType::Atom(super::types::AtomicType::N))
                && matches!(**b, LambekType::Atom(super::types::AtomicType::N))
                && let Sem::Func { word, body } = func
                && let Some(canonical) = super::tokenize::nominal_coordinator_canonical(word)
            {
                return flatten_coordination(canonical, arg, body);
            }
            let func_word = extract_word(func);
            let arg_word = extract_word(arg);
            Sem::Pred {
                word: format!("{} {}", func_word, arg_word),
                role: GrammaticalRole::Argument,
                provenance: PredProvenance::Composite {
                    func: func_word,
                    arg: arg_word,
                },
            }
        }
        // Result is a function type — partial application, still short of an
        // atomic result. ACCUMULATE onto whatever `func` already absorbed
        // (`extract_arguments` returns its existing `body`, empty for a
        // fresh functor) rather than replacing it with only the new `arg` —
        // a 2+-argument functor (a ditransitive verb, a do-support chain
        // like "does X mean") passes through this branch more than once
        // before reaching an atomic result, and the previous absorbed
        // argument must survive the next application.
        LambekType::RightDiv(_, _) | LambekType::LeftDiv(_, _) => {
            // A MEDIAL POST-VERB SUPPLEMENT (defines-lens gap G2 — "means,
            // with respect to Y, Z", the EVV headline shape, 42 U.S.C.
            // § 1396b(l)(5)) is syncategorematic at the TRANSITIVE-VERB
            // level: Steedman's general VP-modifier schema (see
            // `svo::medial_supplement_verb`'s own doc) applied one level
            // BELOW the saturated VP the generic accumulation logic below
            // otherwise assumes every RightDiv/LeftDiv application is
            // building toward. Recognized by `result_type == func_type` at
            // EXACTLY the `transitive_verb` shape — the structural
            // invariant an `X\X` modifier composition always produces (the
            // verb's own type is unchanged by absorbing its modifier) — AND
            // the reserved synthetic marker
            // `tokenize::is_medial_supplement_marker` on the ARGUMENT,
            // mirroring the double guard the NP-result branch above uses:
            // `func` (the still-unabsorbed verb) passes through UNCHANGED,
            // ready to absorb its REAL object next, instead of the
            // supplement being accumulated into `body` as a spurious extra
            // argument (`defines_pointers` requires EXACTLY two).
            if result_type == func_type
                && *func_type == super::types::svo::transitive_verb()
                && let Sem::Func { word, body } = arg
                && body.is_empty()
                && super::tokenize::is_medial_supplement_marker(word)
            {
                return func.clone();
            }
            // A PREPOSITIONAL-VERB PARTICLE (`svo::transitive_verb_particle`,
            // `((NP\S)/NP)\((NP\S)/NP)` — "into" in "enters INTO a
            // qualifying non-binding instrument", 1 U.S.C. § 112b(k)(2)) is
            // ALSO syncategorematic at the TRANSITIVE-VERB level — Huddleston
            // & Pullum (2002) Ch. 7 §3 "Prepositional verbs": the preposition
            // is lexically selected by the verb and contributes no
            // independent content of its own, so the governing verb's OWN
            // `Sem` (its predicate word — "enters", not "into") survives
            // unchanged, ready to absorb its REAL object next. SAME
            // structural collision as the medial-supplement-marker check
            // just above — both reach the IDENTICAL `func_type == result_type
            // == transitive_verb()` shape (`svo::transitive_verb_particle`'s
            // own doc: "the SAME 'X for an X' modifier schema... instantiated
            // here ONE level down") — disambiguated the SAME double-guard way
            // (type shape AND the argument's own surface), just the OPPOSITE
            // surface condition: this branch fires on any ORDINARY (non-
            // marker) fresh preposition leaf, reached only once the
            // reserved-marker check above has already failed to match, so a
            // genuine medial-verb-supplement marker is never swallowed here.
            if result_type == func_type
                && *func_type == super::types::svo::transitive_verb()
                && let Sem::Func { word, body } = arg
                && body.is_empty()
                && !super::tokenize::is_medial_supplement_marker(word)
            {
                return func.clone();
            }
            // The INFINITIVE-MARKER "to" (`svo::infinitive_to`,
            // `(NP\S[to])/(NP\S[b])`) is SYNCATEGOREMATIC — Huddleston &
            // Pullum (2002), *The Cambridge Grammar of the English
            // Language*, Ch. 14 §2: the infinitival particle "to" is a
            // grammatical marker with no semantic content of its own, the
            // SAME closed-class-contributes-nothing status
            // `is_fronted_scope_adjunct_head`'s own pass-through rule
            // already establishes at the clause level (`S/S` above). So
            // once "to" has absorbed its bare-infinitival VP complement
            // (`arg`, already `NP\S[b]`-shaped — possibly via the
            // `S(None)` WILDCARD an ordinary `NP\S` verb reading unifies
            // through, the SAME mechanism `modal_question`/`does_support`
            // already rely on for their own bare-stem-VP complements), the
            // complement's OWN meaning passes through UNCHANGED rather than
            // being wrapped in a spurious `Func{word:"to", body:[...]}` —
            // gated on BOTH the derived TYPE shape (`func_type ==
            // infinitive_to()` exactly) AND the functor's own SURFACE
            // (`tokenize::is_infinitive_marker`), the double guard
            // `svo::nominal_coordinator_np`'s own doc comment establishes
            // as this codebase's standing discipline for a semantically
            // load-bearing dispatch.
            if *func_type == super::types::svo::infinitive_to()
                && super::tokenize::is_infinitive_marker(&extract_predicate(func))
            {
                return arg.clone();
            }
            let predicate = extract_predicate(func);
            let mut body = extract_arguments(func);
            body.push(arg.clone());
            Sem::Func {
                word: predicate,
                body,
            }
        }
        _ => func.clone(),
    }
}

/// Flatten an n-ary nominal coordination (defines-lens gap G4(a)) into ONE
/// [`Sem::Func`] whose `word` is the CANONICAL conjunction and whose `body`
/// lists every conjunct in surface order — the generalized-conjunction
/// treatment of coordination (Partee & Rooth 1983, "Generalized Conjunction
/// and Type Ambiguity," in Bäuerle, Schwarze & von Stechow (eds.), *Meaning,
/// Use, and Interpretation of Language*, de Gruyter, pp. 361-383: a
/// coordinator denotes an operation over its conjuncts, parameterized by the
/// conjunction itself — exactly the shape [`Sem::Func`] already gives a
/// still-unsaturated functor, reused here for a SATURATED coordination
/// rather than inventing a new `Sem` variant, so every existing exhaustive
/// match over `Sem` elsewhere in the codebase stays exhaustive unchanged).
///
/// `arg` is the newly-absorbed LEFT conjunct; `body` is whatever the
/// coordinator's own `(NP\NP)/NP`/`(N\N)/N` functor had already absorbed on
/// its right — EITHER one already-resolved conjunct (a plain two-item "X
/// and Y") OR, for an n-ary list
/// ([`tokenize::find_list_coordinator_commas`]'s iterated-binary-application
/// shape), a NESTED coordination of the SAME canonical conjunction — which
/// is SPLICED FLAT rather than left nested, so a 3+-item list reads as ONE
/// flat list regardless of how many binary reduction steps built it.
fn flatten_coordination(canonical: &str, arg: &Sem, body: &[Sem]) -> Sem {
    let mut items = alloc::vec![arg.clone()];
    for b in body {
        match b {
            Sem::Func { word, body: inner }
                if super::tokenize::nominal_coordinator_canonical(word) == Some(canonical) =>
            {
                items.extend(inner.iter().cloned());
            }
            other => items.push(other.clone()),
        }
    }
    Sem::Func {
        word: canonical.to_string(),
        body: items,
    }
}

/// Flatten a comparison-relation preposition's absorbed object(s) into a
/// flat participant list — the PP-complement counterpart of
/// [`flatten_coordination`] ("the difference BETWEEN X and Y" /
/// "... between X, Y, and Z"), reusing the SAME loaded coordinator oracle
/// ([`tokenize::nominal_coordinator_canonical`]) the NP/N-level coordination
/// branches already use, so an n-ary list (already folded into ONE `Func`
/// upstream by `flatten_coordination`, e.g. the 3-way "pooled income
/// trusts, Miller trusts, and qualifying income trusts") needs no special
/// case here: its single conjunction `Func` is recognized and its `body`
/// spliced in directly. `prep_body` is a plain preposition's absorbed
/// argument list (`svo::preposition`, always exactly one element — see the
/// generic `RightDiv`/`LeftDiv` accumulate branch above), so this degrades
/// to a one-element result for a non-coordinated PP object.
fn flatten_pp_object_conjuncts(prep_body: &[Sem]) -> Vec<Sem> {
    prep_body
        .iter()
        .flat_map(|obj| match obj {
            Sem::Func { word, body }
                if super::tokenize::nominal_coordinator_canonical(word).is_some() =>
            {
                body.clone()
            }
            other => alloc::vec![other.clone()],
        })
        .collect()
}

fn extract_predicate(sem: &Sem) -> String {
    match sem {
        Sem::Pred { word, .. } => word.clone(),
        Sem::Func { word, .. } => word.clone(),
        Sem::Concept { word, .. } => word.clone(),
        Sem::Prop { predicate, .. } | Sem::Question { predicate, .. } => predicate.clone(),
    }
}

fn extract_word(sem: &Sem) -> String {
    match sem {
        Sem::Pred { word, .. } => word.clone(),
        Sem::Func { word, .. } => word.clone(),
        Sem::Concept { word, .. } => word.clone(),
        Sem::Prop { predicate, .. } | Sem::Question { predicate, .. } => predicate.clone(),
    }
}

fn extract_arguments(sem: &Sem) -> Vec<Sem> {
    match sem {
        Sem::Func { body, .. } => body.clone(),
        Sem::Prop { arguments, .. } | Sem::Question { arguments, .. } => arguments.clone(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::composed::ComposedReasoner;
    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::lambek::types::svo;

    /// The degenerate wh-determiner derivation of "what is a long" (leaves
    /// pinned by the R-2 trace-probe: `what:(S[wq]/(NP\S))/N`, `is:N/N`,
    /// `a:N`, `long:NP\S`) interprets to a Question whose lone queried entity
    /// is the MINTED concatenation "is a" — and the N-result composition rule
    /// marks it [`PredProvenance::Composite`], so the answer layer can refuse
    /// to present a non-word as an unknown word (axiom
    /// `DefiniendumIsALexicalUnit` in `pr4xis-chat`).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn n_result_composition_carries_composite_provenance() {
        use crate::cognitive::linguistics::lambek::types::{
            AtomicType, LambekType, SentenceFeature,
        };
        let n = || LambekType::n();
        let np = || LambekType::np();
        let s = |f: Option<SentenceFeature>| LambekType::Atom(AtomicType::S(f));

        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "what".into(),
                lambek_type: LambekType::right_div(
                    LambekType::right_div(
                        s(Some(SentenceFeature::Wq)),
                        LambekType::left_div(np(), s(None)),
                    ),
                    n(),
                ),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "is".into(),
                lambek_type: LambekType::right_div(n(), n()),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "a".into(),
                lambek_type: n(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "long".into(),
                lambek_type: LambekType::left_div(np(), s(None)),
            },
        ];

        let Sem::Question { arguments, .. } = interpret(&tokens, English::sample_static()) else {
            panic!("the degenerate derivation interprets to a Question");
        };
        let composites: alloc::vec::Vec<(&str, &str)> = arguments
            .iter()
            .filter_map(Sem::argument_composite_parts)
            .collect();
        assert_eq!(
            composites,
            alloc::vec![("is", "a")],
            "the lone queried entity is the minted 'is a' with Composite provenance"
        );
    }

    /// Bare nominal (noun-noun) compounding: "consultation" (N/N, the loaded
    /// SECOND Noun row) + "services" (N) composes exactly like an
    /// attributive adjective + noun — `montague::apply`'s generic
    /// `N/N + N → N` branch, unchanged. Hockenmaier & Steedman (2005),
    /// CCGbank User's Manual, MS-CIS-05-09, §3.6.1/§3.6.2.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nominal_compound_composes_via_the_generic_n_result_rule() {
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "consultation".into(),
                lambek_type: svo::nominal_modifier_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "services".into(),
                lambek_type: svo::noun(),
            },
        ];
        let sem = interpret(&tokens, English::sample_static());
        assert_eq!(
            sem.argument_composite_parts(),
            Some(("consultation", "services")),
            "a bare nominal compound concatenates func before arg, Composite \
             provenance, exactly like the adjective+noun case"
        );
    }

    /// RIGHT-HEADEDNESS (Levi 1978; Selkirk 1982, right-headed `[N N]_N`;
    /// Huddleston & Pullum 2002 Ch. 19): for ANY two nominal surfaces, a
    /// nominal-modifier-noun + noun composition preserves English word
    /// order in the minted surface -- the modifier leads, the head (the
    /// rightmost word, "arg") trails -- exactly as the existing adjective
    /// composition already does. Structural property, not lexicon-specific.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn prop_nominal_compound_is_right_headed() {
        use proptest::prelude::*;
        fn arb_lowercase_word() -> impl Strategy<Value = alloc::string::String> {
            proptest::collection::vec(proptest::char::range('a', 'z'), 2..12)
                .prop_map(|chars| chars.into_iter().collect())
        }
        proptest!(|(modifier in arb_lowercase_word(), head in arb_lowercase_word())| {
            let tokens = alloc::vec![
                TypedToken { expression_use: ExpressionUse::Used,
                    word: modifier.clone(),
                    lambek_type: svo::nominal_modifier_noun(),
                },
                TypedToken { expression_use: ExpressionUse::Used,
                    word: head.clone(),
                    lambek_type: svo::noun(),
                },
            ];
            let sem = interpret(&tokens, English::sample_static());
            prop_assert_eq!(
                sem.argument_composite_parts(),
                Some((modifier.as_str(), head.as_str()))
            );
        });
    }

    /// The relational-question SEMANTICS in isolation (no parse plumbing): given
    /// the typed tokens for "is X part of Y", `interpret` lifts the relation from
    /// the COMPLEMENT ("part of") into the `Question` predicate and flattens its
    /// object into the arguments — `Question{ "part of", [X, Y] }`. The lift fires
    /// only because `relation_for_surface("part of")` is loaded (ComposedReasoner
    /// carries the relation lexicon); on plain English it would not.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_x_part_of_y_interprets_to_a_relational_question() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "is".into(),
                lambek_type: svo::question_copula_pred(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "alpha".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "part of".into(),
                lambek_type: svo::relational_predicate(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "beta".into(),
                lambek_type: svo::proper_noun(),
            },
        ];

        match interpret(&tokens, &en) {
            Sem::Question {
                predicate,
                arguments,
                ..
            } => {
                assert_eq!(
                    predicate, "part of",
                    "the relation comes from the complement, not the copula 'is'"
                );
                let names: Vec<String> = arguments.iter().map(extract_entity_name).collect();
                assert_eq!(
                    names,
                    alloc::vec!["alpha".to_string(), "beta".to_string()],
                    "subject and object are flattened into the arguments, in order"
                );
            }
            other => panic!("expected a relational Question, got {other:?}"),
        }
    }

    /// The plain copula "is X a Y" is UNCHANGED: the argument is a bare entity NP
    /// (not a relational `Func`), so the predicate stays the copula "is" (→ the
    /// Subsumption default at dispatch). Proves the lift does not fire spuriously.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_x_a_y_keeps_the_copula_predicate() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "is".into(),
                lambek_type: svo::question_copula(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "alpha".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "beta".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Question { predicate, .. } => assert_eq!(
                predicate, "is",
                "a plain copula question keeps the copula predicate"
            ),
            other => panic!("expected a Question, got {other:?}"),
        }
    }

    /// A predicate-adjective definition question — "what is able" — extracts the
    /// adjective as the single queried definiendum, exactly as the chat's own
    /// `arguments.iter().filter_map(Sem::argument_name)` does. The predicate
    /// adjective `S[adj]\NP` fills the copula's argument slot (Montague; Steedman
    /// 2000 predicate complements), so `argument_name` reaches it. Before the
    /// role fix it lex'd to a dropped `Func{Functor}` and "what is able" yielded
    /// ZERO entities — the adjective-define failure the capability corpus
    /// surfaces at scale. The wh-word "what" and the copula "is" stay Functors,
    /// dropped structurally.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn what_is_able_extracts_the_predicate_adjective_as_the_definiendum() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "what".into(),
                lambek_type: svo::wh_what(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "is".into(),
                lambek_type: svo::copula_adj(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "able".into(),
                lambek_type: svo::predicate_adjective(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Question { arguments, .. } => {
                let entities: Vec<String> =
                    arguments.iter().filter_map(Sem::argument_name).collect();
                assert_eq!(
                    entities,
                    alloc::vec!["able".to_string()],
                    "the predicate adjective is the single queried definiendum; \
                     the wh-word and copula are dropped functors"
                );
            }
            other => panic!("expected a wh Question, got {other:?}"),
        }
    }

    /// TASK #15 (G1): a fronted scope-setting sentential adjunct is
    /// SYNCATEGOREMATIC — Steedman (2000) Ch.2's adjunct/argument
    /// distinction, applied at the CLAUSE level exactly as
    /// `GrammaticalRole::from_lambek_type` already applies it at the NP
    /// level to the copula. Once "the term 'consumer' means a natural
    /// person." (the SAME real, byte-verified declarative
    /// `social::judicial::statute_structure::grounding`'s own test suite
    /// grounds throughout, 15 U.S.C. § 6603(h)(6)(A)) reduces to a complete
    /// `Prop`, prefixing it with a REAL, independently-attested fronted
    /// adjunct of each of the report's four G1 shapes (all four byte-verified
    /// against `usc_title_42-pl-119-90.xml` — see
    /// `grounding::tests::recognizes_the_term_x_means_y_behind_a_fronted_*`
    /// for the exact citations) leaves that SAME `Prop` untouched: the
    /// adjunct is dropped, never becoming a spurious third argument or
    /// hijacking the predicate.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_fronted_scope_adjunct_is_semantically_transparent() {
        use crate::cognitive::linguistics::english::english_loaded;
        use crate::cognitive::linguistics::lambek::reduce::reduce_with_alternatives;
        use crate::cognitive::linguistics::lambek::tokenize;

        let en = english_loaded();
        let tail = "the term \u{201C}consumer\u{201D} means a natural person.";
        for prefix in [
            "For this purpose, ",             // fronted_scope_adjunct_np, "for"
            "In this section, ",              // fronted_scope_adjunct_np, "in"
            "Except for a capital offense, ", // fronted_scope_adjunct_pp, "except"
            "Subject to this part, ",         // fronted_scope_adjunct_pp, "subject"
        ] {
            let text = alloc::format!("{prefix}{tail}");
            let (tokens, alternatives) = tokenize::tokenize_with_alternatives(&text, en);
            let reduction = reduce_with_alternatives(&tokens, &alternatives);
            assert!(
                reduction.success,
                "{prefix:?} must not block the parse; final_type={:?}",
                reduction.final_type
            );
            match interpret(&reduction.remaining, en) {
                Sem::Prop {
                    predicate,
                    arguments,
                } => {
                    assert_eq!(
                        predicate, "means",
                        "{prefix:?}: the predicate must be the REST clause's own verb"
                    );
                    assert_eq!(
                        arguments.len(),
                        2,
                        "{prefix:?}: the adjunct must not add a third argument; got {arguments:?}"
                    );
                    assert_eq!(
                        arguments.last().and_then(Sem::argument_name),
                        Some("consumer".to_string()),
                        "{prefix:?}: the definiendum survives, unaffected by the fronted adjunct"
                    );
                }
                other => panic!("{prefix:?}: expected a Prop, got {other:?}"),
            }
        }
    }

    /// TASK #16 (G2): the POST-VERB medial supplement ("means, with respect
    /// to Y, Z" — the EVV headline shape, 42 U.S.C. § 1396b(l)(5)) is
    /// syncategorematic at the transitive-verb level. The SAME "the term
    /// 'emolument' means compensation" shape
    /// `the_term_x_means_y_extracts_the_apposed_definiendum_as_the_subject`
    /// grounds, with the merged supplement marker spliced in between "means"
    /// and the object: the resulting `Prop` is IDENTICAL — two arguments,
    /// the definiens first, the apposed definiendum last — proving the
    /// verb's fresh, unabsorbed state survives the supplement.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_medial_verb_supplement_is_semantically_transparent() {
        use crate::cognitive::linguistics::lambek::tokenize::MEDIAL_SUPPLEMENT_VERB_MARKER;
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "term".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "emolument".into(),
                lambek_type: svo::close_apposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "means".into(),
                lambek_type: svo::transitive_verb(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: MEDIAL_SUPPLEMENT_VERB_MARKER.into(),
                lambek_type: svo::medial_supplement_verb(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "compensation".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Prop {
                predicate,
                arguments,
            } => {
                assert_eq!(
                    predicate, "means",
                    "the predicate must be the verb's own surface, not the marker"
                );
                assert_eq!(
                    arguments.len(),
                    2,
                    "the supplement must not add a third argument; got {arguments:?}"
                );
                assert_eq!(
                    extract_entity_name(arguments.last().expect("two arguments")),
                    "emolument",
                    "the subject survives, unaffected by the post-verb supplement"
                );
                assert_eq!(
                    extract_entity_name(&arguments[0]),
                    "compensation",
                    "the object survives, unaffected by the post-verb supplement"
                );
            }
            other => panic!("expected a Prop, got {other:?}"),
        }
    }

    /// TASK #16 (G2): the SUBJECT-VERB medial supplement ("the term 'X',
    /// used with respect to Y, means Z" / "the term 'X', as used in this
    /// title, means Z") is syncategorematic at the NP level — the mirror
    /// image of [`a_medial_verb_supplement_is_semantically_transparent`],
    /// interrupting BEFORE the verb instead of after.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_medial_subject_verb_supplement_is_semantically_transparent() {
        use crate::cognitive::linguistics::lambek::tokenize::MEDIAL_SUPPLEMENT_NP_MARKER;
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "term".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "emolument".into(),
                lambek_type: svo::close_apposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: MEDIAL_SUPPLEMENT_NP_MARKER.into(),
                lambek_type: svo::medial_supplement_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "means".into(),
                lambek_type: svo::transitive_verb(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "compensation".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Prop {
                predicate,
                arguments,
            } => {
                assert_eq!(predicate, "means");
                assert_eq!(
                    arguments.len(),
                    2,
                    "the supplement must not add a third argument; got {arguments:?}"
                );
                assert_eq!(
                    extract_entity_name(arguments.last().expect("two arguments")),
                    "emolument",
                    "the apposed definiendum survives, unaffected by the \
                     subject-verb supplement"
                );
                assert_eq!(
                    extract_entity_name(&arguments[0]),
                    "compensation",
                    "the object survives, unaffected by the subject-verb supplement"
                );
            }
            other => panic!("expected a Prop, got {other:?}"),
        }
    }

    /// The CONFUSABLE shape a medial NP-supplement must NOT hijack: the
    /// SAME `NP\NP` shape [`close_apposition`] uses is a genuine, EXPECTED
    /// collision (see `svo::medial_supplement_np`'s own doc) — a real
    /// close-apposition leaf (word "emolument", never the reserved marker)
    /// must keep taking over reference exactly as
    /// `close_apposition_promotes_the_quoted_definiendum_over_the_head_noun`
    /// already proves, unaffected by the new guard being checked first.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_genuine_close_apposition_leaf_is_not_mistaken_for_a_medial_supplement() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "term".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "emolument".into(),
                lambek_type: svo::close_apposition(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Concept { word, .. } => {
                assert_eq!(
                    word, "emolument",
                    "a genuine close-apposition leaf still promotes the quoted \
                     definiendum, unaffected by the new medial-supplement guard"
                );
            }
            other => panic!("expected a Concept, got {other:?}"),
        }
    }

    fn extract_entity_name(sem: &Sem) -> String {
        match sem {
            Sem::Concept { word, .. } | Sem::Pred { word, .. } => word.clone(),
            _ => String::new(),
        }
    }

    // ---- defines-lens gap G4(a): nominal coordination semantics ----

    /// A plain two-item NP coordination, "member or parent" — the SAME
    /// `NP\NP` shape [`close_apposition`]/[`medial_supplement_np`] already
    /// share, disambiguated here by the functor's own coordinator surface.
    /// Before this task the generic NP-result fallback silently DROPPED the
    /// right conjunct, returning just the left one; now it builds a real
    /// `Sem::Func{word:"or", body:[member, parent]}` — both conjuncts
    /// survive, in surface order.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_two_item_np_coordination_keeps_both_conjuncts() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "member".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "or".into(),
                lambek_type: svo::nominal_coordinator_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "parent".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Func { word, body } => {
                assert_eq!(word, "or");
                assert_eq!(
                    body.iter().map(extract_entity_name).collect::<Vec<_>>(),
                    alloc::vec!["member".to_string(), "parent".to_string()]
                );
            }
            other => panic!("expected a coordination Func, got {other:?}"),
        }
    }

    /// The REAL report-cited n-ary shape, 42 U.S.C. § 300ii(5): "an unpaid
    /// family member, a foster parent, or another unpaid individual" — one
    /// literal "or" plus one
    /// [`tokenize::LIST_COORDINATOR_MARKER_OR`]-minted comma coordinator
    /// (`tokenize::find_list_coordinator_commas`'s own output, spliced in
    /// here as the tokenizer would). Proves the FLATTENING half of
    /// `flatten_coordination`: a 3-item list built by TWO binary reduction
    /// steps still yields ONE flat `Func` with all three conjuncts, in
    /// surface order, not a nested `Func`-inside-`Func`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_n_ary_np_coordination_via_a_list_coordinator_marker_flattens_to_one_func() {
        use crate::cognitive::linguistics::lambek::tokenize::LIST_COORDINATOR_MARKER_OR;
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "member".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: LIST_COORDINATOR_MARKER_OR.into(),
                lambek_type: svo::nominal_coordinator_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "parent".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "or".into(),
                lambek_type: svo::nominal_coordinator_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "individual".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Func { word, body } => {
                assert_eq!(word, "or");
                assert_eq!(
                    body.iter().map(extract_entity_name).collect::<Vec<_>>(),
                    alloc::vec![
                        "member".to_string(),
                        "parent".to_string(),
                        "individual".to_string()
                    ],
                    "a flat 3-item list, not a nested Func-inside-Func; got {body:?}"
                );
            }
            other => panic!("expected a flat coordination Func, got {other:?}"),
        }
    }

    /// The REAL report-cited example, 42 U.S.C. § 3002(42): "bodily injury,
    /// impairment, or disease" — coordination at the COMMON-NOUN level
    /// (`svo::nominal_coordinator_n`, `(N\N)/N`). Before this task the
    /// N-result branch unconditionally treated ANY `N\N + N -> N`
    /// composition as a modifier+noun concatenation (the shape adjective
    /// composition uses), which would have mangled a coordination into a
    /// garbled composite surface; now it is recognized and flattened
    /// exactly like the NP-level case.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_n_level_coordination_flattens_instead_of_composite_concatenating() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "impairment".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "or".into(),
                lambek_type: svo::nominal_coordinator_n(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "disease".into(),
                lambek_type: svo::noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Func { word, body } => {
                assert_eq!(word, "or");
                assert_eq!(
                    body.iter().map(extract_entity_name).collect::<Vec<_>>(),
                    alloc::vec!["impairment".to_string(), "disease".to_string()]
                );
            }
            other => panic!("expected a coordination Func, got {other:?}"),
        }
    }

    /// Close apposition ("the term 'X'") in isolation: `the:NP/N + term:N →
    /// NP` (the ordinary determiner+noun composition, unaffected), then
    /// `[the term]:NP + emolument:NP\NP → NP` — the quoted definiendum's own
    /// concept ("emolument") takes over reference; the head noun "term"
    /// contributes nothing further. Before this fix the NP-result branch
    /// dispatched on `result_type` alone (both compositions produce NP), so
    /// it could not distinguish the two shapes and would have returned
    /// "term" (the second composition's `arg`), not the quoted definiendum.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn close_apposition_promotes_the_quoted_definiendum_over_the_head_noun() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "term".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "emolument".into(),
                lambek_type: svo::close_apposition(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Concept { word, .. } => {
                assert_eq!(
                    word, "emolument",
                    "the quoted definiendum's concept takes over reference, \
                     not the head noun 'term'"
                );
            }
            other => panic!("expected a Concept, got {other:?}"),
        }
    }

    /// The defines-lens gap G5 shape: "the terms 'consumer' and
    /// 'individual' mean ..." — TWO quoted spans coordinated by the
    /// RESERVED apposition-coordinator marker (never a literal "and")
    /// modify "the terms" as ONE combined apposition, dropping the head
    /// NP entirely and surviving as a `Sem::Func` whose `body` holds BOTH
    /// promoted `Sem::Concept`s.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn coordinated_close_apposition_definienda_drop_the_terms_and_keep_both_concepts() {
        use crate::cognitive::linguistics::lambek::tokenize::APPOSITION_COORDINATOR_MARKER_AND;
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "terms".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "consumer".into(),
                lambek_type: svo::close_apposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: APPOSITION_COORDINATOR_MARKER_AND.into(),
                lambek_type: svo::nominal_coordinator_apposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "individual".into(),
                lambek_type: svo::close_apposition(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Func { word, body } => {
                assert_eq!(
                    word, APPOSITION_COORDINATOR_MARKER_AND,
                    "the result keeps the MARKER as its own word (never canonicalized \
                     to plain \"and\") — the signal `grounding::definiendum_words` \
                     needs downstream"
                );
                let names: alloc::collections::BTreeSet<String> =
                    body.iter().map(extract_entity_name).collect();
                assert_eq!(
                    names,
                    alloc::collections::BTreeSet::from([
                        "consumer".to_string(),
                        "individual".to_string()
                    ]),
                    "both coordinated definienda survive as Concepts; got {body:?}"
                );
            }
            other => panic!("expected a coordination Func, got {other:?}"),
        }
    }

    /// The CONFUSABLE shape close apposition must NOT hijack: "the Secretary
    /// of Commerce". `svo::preposition` is `(NP\NP)/NP`, so "of Commerce"
    /// reduces to the SAME bare `NP\NP` type `close_apposition` uses once it
    /// has absorbed its object "Commerce" — and would then reach the exact
    /// same NP-result branch as "the Secretary" + [of Commerce] → NP. Before
    /// the `body.is_empty()` refinement, guarding on type shape ALONE would
    /// have wrongly promoted the PREPOSITION's own surface ("of") as the
    /// referent. The correct (pre-existing, UNCHANGED) behavior: the PP is
    /// semantically transparent here — the head NP's own concept
    /// ("secretary") survives, exactly as it did before this change.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_derived_prepositional_np_np_does_not_trigger_the_apposition_guard() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "secretary".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "of".into(),
                lambek_type: svo::preposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "commerce".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Concept { word, .. } => {
                assert_eq!(
                    word, "secretary",
                    "a derived (non-leaf) NP\\NP must not be mistaken for a \
                     close-apposition leaf; the PP-modified head NP's own \
                     concept survives unchanged, exactly as before this fix"
                );
            }
            other => panic!("expected a Concept, got {other:?}"),
        }
    }

    /// Construction 2: "the difference between the individual budget and
    /// the spending plan" — a DERIVED relational noun ("difference",
    /// Barker 2011 §1.4) overtly expressing BOTH participants via its own
    /// PP complement, rather than the pre-existing bare-`Concept{"difference"}`
    /// gloss-dump the un-fixed guard produced.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_difference_between_np_and_np_extracts_both_arguments_as_a_relation() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "difference".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "between".into(),
                lambek_type: svo::preposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "budget".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "and".into(),
                lambek_type: svo::nominal_coordinator_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "plan".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Func { word, body } => {
                assert_eq!(word, "difference");
                assert_eq!(
                    body.iter().map(extract_entity_name).collect::<Vec<_>>(),
                    alloc::vec!["budget".to_string(), "plan".to_string()],
                    "both PP-complement participants survive, in surface order; got {body:?}"
                );
            }
            other => panic!("expected a comparison-relation Func, got {other:?}"),
        }
    }

    /// The row-1511 shape: "the difference between A, B, and C" — the
    /// PP-complement's list-coordinator marker (already folded into ONE
    /// `Func` upstream by `flatten_coordination`) is spliced flat into the
    /// comparison relation's own body by `flatten_pp_object_conjuncts`,
    /// never left nested.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_difference_between_a_three_way_list_flattens_all_three_conjuncts() {
        use crate::cognitive::linguistics::lambek::tokenize::LIST_COORDINATOR_MARKER_AND;
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "difference".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "between".into(),
                lambek_type: svo::preposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "alpha".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: LIST_COORDINATOR_MARKER_AND.into(),
                lambek_type: svo::nominal_coordinator_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "beta".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "and".into(),
                lambek_type: svo::nominal_coordinator_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "gamma".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Func { word, body } => {
                assert_eq!(word, "difference");
                assert_eq!(
                    body.iter().map(extract_entity_name).collect::<Vec<_>>(),
                    alloc::vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
                    "a 3-way list flattens to ONE flat body, not nested; got {body:?}"
                );
            }
            other => panic!("expected a comparison-relation Func, got {other:?}"),
        }
    }

    /// The direct regression pin for the "Secretary of Commerce" guard
    /// (§4 of the design): the SAME `NP\NP` + coordinated-PP-object shape
    /// as the two tests above, but with an UNREGISTERED head ("secretary"
    /// substituted for "difference") — must fall all the way through to
    /// `n_to_np_argument(arg)`, exactly as before this construction, never
    /// promoting the coordinated PP object.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_difference_between_np_still_falls_through_for_an_unregistered_head() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "secretary".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "between".into(),
                lambek_type: svo::preposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "budget".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "and".into(),
                lambek_type: svo::nominal_coordinator_np(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "plan".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Concept { word, .. } => {
                assert_eq!(
                    word, "secretary",
                    "an unregistered head must fall through unchanged, exactly \
                     as the pre-existing Secretary-of-Commerce behavior"
                );
            }
            other => panic!("expected a Concept, got {other:?}"),
        }
    }

    /// The full declarative shape close apposition exists for: "the term
    /// 'emolument' means compensation" — the subject NP is the
    /// apposition-promoted quoted definiendum ("emolument"), absorbed LAST
    /// via backward application (the same absorption-order convention
    /// `montague_ditransitive_verb_keeps_every_absorbed_argument` proves:
    /// objects first, subject last) — so `arguments.last()` is the
    /// definiendum, `arguments[0]` the definiens, never the reverse.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_term_x_means_y_extracts_the_apposed_definiendum_as_the_subject() {
        let en = ComposedReasoner::new(English::sample_static(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "the".into(),
                lambek_type: svo::determiner(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "term".into(),
                lambek_type: svo::noun(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "emolument".into(),
                lambek_type: svo::close_apposition(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "means".into(),
                lambek_type: svo::transitive_verb(),
            },
            TypedToken {
                expression_use: ExpressionUse::Used,
                word: "compensation".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Prop {
                predicate,
                arguments,
            } => {
                assert_eq!(predicate, "means");
                assert_eq!(arguments.len(), 2);
                assert_eq!(
                    extract_entity_name(arguments.last().expect("two arguments")),
                    "emolument",
                    "the subject (absorbed last) is the apposed definiendum, not 'term'"
                );
                assert_eq!(
                    extract_entity_name(&arguments[0]),
                    "compensation",
                    "the object (absorbed first) is the definiens"
                );
            }
            other => panic!("expected a Prop, got {other:?}"),
        }
    }
}
