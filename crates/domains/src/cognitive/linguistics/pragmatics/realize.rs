#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::response::ResponseFrame;
use crate::social::software::markup::xml::lmf::ontology::LmfPos;
use pr4xis::ontology::meta::OntologyName;

// Response realization — the Levelt production pipeline.
//
// This is the RIGHT ADJOINT of parsing (the generation functor).
// Parse: Text → Syntax → Semantics (left adjoint, F)
// Generate: Semantics → Syntax → Text (right adjoint, G)
//
// Levelt (1989): Conceptualizer → PreverbalMessage → Formulator → SurfaceForm
//
// The ResponseFrame is the CommunicativeGoal (Appelt 1985).
// The ResponseContent is the PreverbalMessage.
// The SentencePlan composes words in SVO order.
// The SurfaceForm is the realized text.
//
// de Groote (2001): generation in categorial grammar = beta-reduction.
// We compose NP + (NP\S)/NP + NP → S for assertions,
// (S/NP)/NP + NP + NP → S[q] for questions.
//
// References:
// - Levelt, "Speaking" (1989) — the production model
// - Reiter & Dale, "Building NLG Systems" (2000) — pipeline
// - de Groote, "Towards Abstract Categorial Grammars" (2001)
// - Appelt, "Planning English Sentences" (1985)

/// Pre-extracted presentation of a [`ResponseFrame::RuleAwaitingFact`]
/// answer — built by the CALLER (which holds the typed judicial data:
/// `SourceTextRef.text`, `PinpointCite::to_bluebook()`), never by this
/// domain-agnostic NLG realization layer itself. Keeps `pragmatics` free of
/// a `social::judicial` import — every other cross-domain field on
/// [`ResponseContent`] is already a plain `String`/`Vec<String>`, and this
/// follows the same discipline rather than accepting the typed
/// `ConditionalRule`/`AppliedElement` payload directly.
#[derive(Debug, Clone)]
pub struct ConditionalPresentation {
    pub rule_name: String,
    pub citation: String,
    pub rule_text: String,
    pub missing_facts: Vec<String>,
}

/// Pre-extracted presentation of a [`ResponseFrame::RuleApplies`] /
/// [`ResponseFrame::RuleDoesNotApply`] answer — the multi-turn slot-filling
/// layer's (task #17) terminal turn, once every `Required` element has a
/// [`SlotState`](crate::social::judicial::conditional_rule::SlotState). No
/// `missing_facts` field: unlike [`ConditionalPresentation`], nothing is
/// still outstanding by the time this frame is realized.
#[derive(Debug, Clone)]
pub struct RuleVerdictPresentation {
    pub rule_name: String,
    pub citation: String,
    pub rule_text: String,
}

/// A structured response before surface realization.
/// This is the PreverbalMessage in Levelt's model.
#[derive(Debug, Clone)]
pub struct ResponseContent {
    pub frame: ResponseFrame,
    pub predicate: Option<String>,
    pub entities: Vec<String>,
    pub definitions: Vec<(String, String)>,
    /// A self-model capability enumeration (SelfModel `Capability`
    /// concept) — the loaded ontology/domain names the system can reason
    /// about, realized as a generated list sentence rather than a
    /// structural dump. Empty for every non-capability-query response.
    pub capabilities: Vec<String>,
    /// The [`ResponseFrame::RuleAwaitingFact`] payload. `None` for every
    /// other frame — never read outside `realize_rule_awaiting_fact`.
    pub conditional: Option<ConditionalPresentation>,
    /// The [`ResponseFrame::RuleApplies`] / [`ResponseFrame::RuleDoesNotApply`]
    /// payload. `None` for every other frame — never read outside
    /// `realize_rule_applies`/`realize_rule_does_not_apply`.
    pub verdict: Option<RuleVerdictPresentation>,
}

impl ResponseContent {
    pub fn new(frame: ResponseFrame) -> Self {
        Self {
            frame,
            predicate: None,
            entities: Vec::new(),
            definitions: Vec::new(),
            capabilities: Vec::new(),
            conditional: None,
            verdict: None,
        }
    }

    pub fn with_predicate(mut self, pred: &str) -> Self {
        self.predicate = Some(pred.into());
        self
    }

    pub fn with_entity(mut self, entity: &str) -> Self {
        self.entities.push(entity.into());
        self
    }

    pub fn with_definition(mut self, entity: &str, definition: &str) -> Self {
        self.definitions.push((entity.into(), definition.into()));
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_conditional(mut self, presentation: ConditionalPresentation) -> Self {
        self.conditional = Some(presentation);
        self
    }

    pub fn with_verdict(mut self, presentation: RuleVerdictPresentation) -> Self {
        self.verdict = Some(presentation);
        self
    }
}

/// Realize a ResponseContent into surface text.
///
/// Levelt pipeline: Frame (goal) → Content (message) → Plan (syntax) → Surface (text).
pub fn realize(content: &ResponseContent) -> String {
    match content.frame {
        ResponseFrame::AssertKnowledge => realize_assertion(content),
        ResponseFrame::UnknownVocabulary => realize_gap(content),
        ResponseFrame::UnprovenRelation => realize_abstain(content),
        ResponseFrame::SuggestInterpretation => realize_suggestion(content),
        ResponseFrame::AdmitLimitation => realize_limitation(content),
        ResponseFrame::PhaticReturn => realize_phatic_return(),
        ResponseFrame::RuleAwaitingFact => realize_rule_awaiting_fact(content),
        ResponseFrame::RuleApplies => realize_rule_applies(content),
        ResponseFrame::RuleDoesNotApply => realize_rule_does_not_apply(content),
        ResponseFrame::Comparison => realize_comparison(content),
    }
}

/// Realize a plain-language EXPLANATION of a turn's outcome — the "Why?" layer
/// (doc §5.2). This is the SECOND realization of the same communicative goal
/// ([`ResponseFrame`]): where [`realize`] produces the answer surface, this
/// produces a plain-language gloss of WHY the engine reached that outcome, filled
/// from the LOADED explain-frames table
/// ([`explain_frames`](super::explain_frames), Reiter & Dale 2000 realization
/// stage) and the ACTUAL outcome data of THIS turn — the loaded sources it
/// reasoned over, the surfaces it could not resolve — never a static description
/// of how the pipeline "works" (the why-layer critique's backwards-pipeline
/// hazard).
///
/// Returns `None` for the frames whose PRIMARY response already carries its own
/// reason (`UnprovenRelation`, `RuleAwaitingFact`, `RuleApplies`,
/// `RuleDoesNotApply`, `Comparison`, `PhaticReturn`): the renderer then shows no
/// Why? panel, since a second sentence would only repeat the answer.
///
/// - `frame` — the communicative goal that produced the response.
/// - `answered` — whether the turn grounded an answer (`from_ontology`); an
///   `AssertKnowledge` frame that did NOT ground an answer (a degenerate
///   fallback) gets no why, since there is no honest provenance to state.
/// - `reasoned_over` — the loaded ontologies the answer drew on. The honest
///   three-way provenance split: EMPTY → the compiled built-in substrate;
///   a NAMED built-in source → its plain label; an UNNAMED runtime-loaded source
///   → "your loaded documents" — the latter two projected through
///   [`ontology_labels`](super::ontology_labels) so no raw `OntologyName` leaks.
/// - `unresolved` — the "asked but not loaded" surfaces (the abstention's own
///   `unresolved` set) naming the `{terms}` a vocabulary-gap could not connect.
pub fn realize_why(
    frame: ResponseFrame,
    answered: bool,
    reasoned_over: &[OntologyName],
    unresolved: &[String],
) -> Option<String> {
    use super::explain_frames as ef;
    match frame {
        // Self-explaining frames: the primary response already states its reason,
        // so a Why? sentence would only repeat it.
        ResponseFrame::UnprovenRelation
        | ResponseFrame::RuleAwaitingFact
        | ResponseFrame::RuleApplies
        | ResponseFrame::RuleDoesNotApply
        | ResponseFrame::Comparison
        | ResponseFrame::PhaticReturn => None,
        // Any non-self-explaining frame that GROUNDED an answer: name the
        // provenance from the ACTUAL sources reasoned over. Empty → the compiled
        // substrate; non-empty → the plain-label-projected loaded sources.
        _ if answered => {
            if reasoned_over.is_empty() {
                ef::template(ef::ASSERT_KNOWLEDGE_UNGROUNDED)
            } else {
                let sources = super::ontology_labels::project_join(reasoned_over);
                ef::template(ef::ASSERT_KNOWLEDGE_GROUNDED)
                    .map(|t| t.replace("{sources}", &sources))
            }
        }
        // Abstained — the plain gloss per remaining frame, from the turn's own
        // unresolved surfaces (never a canned "I don't know").
        ResponseFrame::UnknownVocabulary => {
            let terms: Vec<&str> = unresolved.iter().map(String::as_str).collect();
            if terms.is_empty() {
                ef::template(ef::UNKNOWN_VOCABULARY_UNTERMED)
            } else {
                let joined = english_list_quoted(&terms);
                ef::template(ef::UNKNOWN_VOCABULARY).map(|t| t.replace("{terms}", &joined))
            }
        }
        ResponseFrame::SuggestInterpretation => ef::template(ef::SUGGEST_INTERPRETATION),
        ResponseFrame::AdmitLimitation => ef::template(ef::ADMIT_LIMITATION),
        // AssertKnowledge that did NOT ground an answer (a degenerate fallback
        // realized from the assertion frame but marked not-from-ontology): no
        // honest provenance to state, so no Why? panel.
        ResponseFrame::AssertKnowledge => None,
    }
}

/// ONE definition's documentary provenance: the loaded lexicon whose gloss the
/// answer recited, and the resource that lexicon's author derived the gloss
/// FROM.
///
/// This is categorically NOT [`realize_why`]'s `reasoned_over`. That names the
/// ontologies this turn opened; this names a document the turn did not
/// necessarily open at all — an authorship claim the lexicon makes about its own
/// text. Conflating them is the exact defect this type exists to make
/// impossible: an answer whose gloss ended in "— 42 USC 300ii(7)" read as
/// statute-backed while only a lexicon entry had been consulted.
///
/// The lexicon is the typed [`OntologyName`] (so it projects through
/// [`ontology_labels`](super::ontology_labels) exactly as `reasoned_over` does
/// and no raw internal name can leak); the citation is the loaded archive's own
/// `dcterms:BibliographicResource` node name, carried verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionProvenance {
    /// The loaded lexicon carrying the gloss.
    pub lexicon: OntologyName,
    /// The documentary resource the gloss was authored from, as the lexicon
    /// declares it (`"42 USC 300ii(7)"`).
    pub citation: String,
}

/// The realized definition-provenance line: the loaded LABEL naming the channel
/// and the realized SENTENCE stating it. Both come from the loaded
/// explain-frames table, so a renderer displays engine-realized language and
/// never writes its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizedProvenance {
    /// The channel's own name ("Definition authored from") — the label a
    /// provenance panel puts beside `detail`.
    pub label: String,
    /// The full sentence naming the lexicon, its citations, and the distinction
    /// from the documents actually read.
    pub detail: String,
}

/// Realize the DEFINITION-PROVENANCE line for a turn — the third, genuinely
/// distinct provenance channel, beside [`realize_why`]'s "answered from
/// {sources}" and the turn's list of loaded ontologies.
///
/// `None` when the turn recited no definition that declares a source — the
/// honest empty case (a WordNet gloss cites no document, and inventing one would
/// be the fabricated-provenance failure the whole channel guards). Deduplicated:
/// two senses of one term authored from the same statute state it once.
pub fn realize_definition_provenance(
    sources: &[DefinitionProvenance],
) -> Option<RealizedProvenance> {
    use super::explain_frames as ef;
    if sources.is_empty() {
        return None;
    }
    // The lexicons, projected to plain labels through the SAME lexicalization
    // `{sources}` already uses in `realize_why` — `project_join` dedups.
    let lexicons: Vec<OntologyName> = sources.iter().map(|s| s.lexicon.clone()).collect();
    let mut citations: Vec<&str> = Vec::new();
    for s in sources {
        if !citations.contains(&s.citation.as_str()) {
            citations.push(&s.citation);
        }
    }
    Some(RealizedProvenance {
        label: ef::template(ef::DEFINITION_AUTHORED_FROM_LABEL)?,
        detail: ef::template(ef::DEFINITION_AUTHORED_FROM)?
            .replace(
                "{sources}",
                &super::ontology_labels::project_join(&lexicons),
            )
            .replace("{citations}", &english_list(&citations)),
    })
}

/// Join surfaces into a quoted English list ("\"a\"", "\"a\" and \"b\"",
/// "\"a\", \"b\", and \"c\"") for the vocabulary-gap `{terms}` slot — the same
/// list grammar [`ontology_labels::project_join`](super::ontology_labels::project_join)
/// uses for `{sources}`. Each surface is quoted (it is a mentioned word, not a
/// used one).
fn english_list_quoted(items: &[&str]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| format!("\"{s}\"")).collect();
    english_list(&quoted.iter().map(String::as_str).collect::<Vec<_>>())
}

/// The bare English list grammar ("a", "a and b", "a, b, and c") — factored out
/// of [`english_list_quoted`] because a CITATION is used, not mentioned: quoting
/// "42 USC 300ii(7)" would read as scare-quoting an authority.
fn english_list(items: &[&str]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].to_string(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let last = items.last().copied().unwrap_or_default();
            format!("{}, and {}", items[..items.len() - 1].join(", "), last)
        }
    }
}

// ---- Sentence planning: compose from semantic roles ----
//
// SVO grammar: Subject + Verb + Object
// Copula sentences: NP + cop + NP → S[dcl]
// Existential: NP + cop + Det + N → S[dcl]
// Negation: NP + cop + "not" + Det + N → S[dcl]

/// Build a copula sentence: "{subject} is {complement}"
/// SVO: NP + (NP\S)/NP + NP → S
pub fn sentence_copula(subject: &str, complement: &str) -> String {
    let det = select_determiner(subject);
    let det2 = select_determiner(complement);
    let cop = select_copula(subject);
    format!("{det}{subject} {cop} {det2}{complement}")
}

/// Build a negative copula sentence: "{subject} is not {complement}"
fn sentence_copula_neg(subject: &str, complement: &str) -> String {
    let det = select_determiner(subject);
    let det2 = select_determiner(complement);
    let cop = select_copula(subject);
    format!("{det}{subject} {cop} not {det2}{complement}")
}

/// Build a RELATIONAL copula sentence: "{subject} is {connective} {complement}" —
/// the `connective` is the relation's LOADED natural surface ("part of"), so a
/// Parthood affirmation reads "a subsection is part of a section", not "is a".
/// `None` is the is-a default and reduces to [`sentence_copula`] ("is a"). The
/// connective is loaded data threaded in by the caller, NOT a hardcoded relation
/// string here.
pub fn sentence_relation(subject: &str, complement: &str, connective: Option<&str>) -> String {
    match connective {
        None => sentence_copula(subject, complement),
        Some(conn) => {
            let det = select_determiner(subject);
            let det2 = select_determiner(complement);
            let cop = select_copula(subject);
            format!("{det}{subject} {cop} {conn} {det2}{complement}")
        }
    }
}

/// The negative form of [`sentence_relation`]: "{subject} is not {connective}
/// {complement}" (a Parthood denial reads "is not part of"). `None` reduces to
/// the is-a negation ("is not a").
fn sentence_relation_neg(subject: &str, complement: &str, connective: Option<&str>) -> String {
    match connective {
        None => sentence_copula_neg(subject, complement),
        Some(conn) => {
            let det = select_determiner(subject);
            let det2 = select_determiner(complement);
            let cop = select_copula(subject);
            format!("{det}{subject} {cop} not {conn} {det2}{complement}")
        }
    }
}

/// Realize the transitivity LICENSING clause of a multi-hop affirmative answer —
/// the *rule* that authorized crossing two or more rungs, not the witness chain:
/// "and is-a is transitive (Tarski (1941) …)". Every part is loaded data the
/// caller read from the Relations ontology (the relation's surface, the
/// `Transitive` structural-property label, its citation) — never composed here.
/// When `citation` is empty (a transitive kind with no dedicated citation axiom
/// yet), the property is still surfaced and the citation is silently omitted, so
/// the caller can name the rule and treat the citation as a documented follow-up.
pub fn sentence_transitivity_license(
    relation_surface: &str,
    property: &str,
    citation: &str,
) -> String {
    if citation.is_empty() {
        format!("and {relation_surface} is {property}")
    } else {
        format!("and {relation_surface} is {property} ({citation})")
    }
}

/// Build a copula question: "is {subject} {complement}?"
/// Inversion: (S[q]/NP)/NP + NP + NP → S[q]
fn sentence_question(subject: &str, complement: &str) -> String {
    let det = select_determiner(subject);
    let det2 = select_determiner(complement);
    let cop = select_copula(subject);
    format!("{cop} {det}{subject} {det2}{complement}?")
}

/// Select determiner: "a" for common nouns, empty for proper/mass nouns.
/// Morphology: "a" before consonant, "an" before vowel.
fn select_determiner(word: &str) -> &'static str {
    if word.is_empty() {
        return "";
    }
    let first = word.chars().next().unwrap_or('x');
    if first.is_uppercase() {
        // Proper noun — no determiner
        ""
    } else if "aeiou".contains(first) {
        "an "
    } else {
        "a "
    }
}

/// Select copula form based on subject.
fn select_copula(_subject: &str) -> &'static str {
    "is" // Default 3rd person singular present
}

/// The determiner a complement of the given part of speech takes. Grammatically
/// the article is a NOMINAL specifier (Quirk et al. 1985 §5.2; Huddleston &
/// Pullum 2002 ch. 5), so a predicate adjective is realized BARE ("what is
/// able", never the ungrammatical "*what is an able") — which also lets the
/// parser type the bare adjective as a predicate adjective (`S[adj]\NP`) rather
/// than a determined noun, the reading the montague extractor needs.
///
/// A VERB slot is realized bare too — its surface is the gerundial
/// nominalization ([`nominal_slot`]), and verbal gerunds reject articles
/// (Huddleston & Pullum 2002 p. 1189, via page-agreeing fetched secondaries;
/// "what is hyperventilating", never "*what is a hyperventilating"). Landed
/// TOGETHER with the gerund extraction path (the FIX-B law: a determiner
/// correction ships with its extraction half or the corpus gate rejects it —
/// the adjective slice measured that lesson at +141 define failures).
///
/// The adverb keeps the existing [`select_determiner`] selection: its frame +
/// extraction path remain a generate-side probe the capability suite still
/// surfaces, exactly as the mass-noun count-vs-mass determiner does.
fn select_determiner_for_pos(word: &str, pos: LmfPos) -> &'static str {
    match pos {
        // Both the head adjective (`a`) and the satellite adjective (`s`) are
        // adjectives — the tokenizer types both as the same adjective category,
        // so both parse as a predicate adjective when realized bare.
        LmfPos::Adjective | LmfPos::SatelliteAdjective => "",
        // A GERUNDIZED verb slot is bare (CGEL p. 1189: verbal gerunds reject
        // articles). A phrasal-verb slot is NOT gerundized (see
        // [`nominal_slot`]) and keeps the collapsed-nominal determiner the
        // multi-word slice measured green — the branch condition is the same
        // single-word test `nominal_slot` gerundizes under.
        LmfPos::Verb if !word.contains(' ') => "",
        _ => select_determiner(word),
    }
}

/// The SURFACE a concept label takes in a nominal slot of a realized frame —
/// the label itself for every part of speech except the verb, whose nominal
/// slot is the gerundial nominalization (`ing_form`, the dual-route Pinker
/// generation over the loaded AGID exceptions): questions ABOUT a verb concept
/// are asked over its -ing nominal ("is coughing exhaling?"), the form CGEL
/// records as serving subject and predicative-complement functions
/// (Huddleston & Pullum 2002 pp. 1220–1222; Fellbaum 1998: verb hypernymy is
/// troponymy, tested over gerundial/infinitival nominalizations).
pub fn nominal_slot(label: &str, pos: LmfPos) -> String {
    use crate::cognitive::linguistics::morphology::english::generation::ing_form;
    match pos {
        // Gerundization is scoped to SINGLE-WORD verb lemmas. A phrasal verb
        // ("care for", "get rid of") inflects on its head, but its gerund
        // surface ("caring for") is not in the lexicon's multi-word surface
        // index, so the collapsed-nominal frame the multi-word slice already
        // measured green ("is a care for …", the FIX-D dual reading) stays —
        // whole-phrase suffixation ("care foring") is garbage the corpus gate
        // rejected (+188 non-edge). Head-inflected phrasal gerunds are a
        // follow-up gated on the surface index carrying inflected phrases.
        LmfPos::Verb if !label.contains(' ') => ing_form(label),
        _ => label.to_string(),
    }
}

/// Build a wh-definition question: "what {cop} {det}{label}" (e.g. "what is a
/// dog", "what is able"). The interrogative "what" and the copula are the SAME
/// grammatical morphemes `sentence_question` composes its polar "is …?"
/// surface from — realizing a definition-seeking `SpeechAct::Question` (Reiter &
/// Dale 2000, realization stage). The `label` is the only content; it is
/// supplied by the caller from a concept's own lemma, never authored here. The
/// determiner is chosen from the concept's part of speech
/// (`select_determiner_for_pos`): an adjective is realized bare, every other
/// POS keeps the existing article selection (the verb/adverb and count-vs-mass
/// determiners remain generate-side probes the capability suite still surfaces).
pub fn wh_definition(label: &str, pos: LmfPos) -> String {
    let surface = nominal_slot(label, pos);
    let det = select_determiner_for_pos(&surface, pos);
    let cop = select_copula(&surface);
    format!("what {cop} {det}{surface}")
}

/// Build a wh-mean question: "what does \u{201C}{label}\u{201D} mean" (Slice B,
/// `.notes/chat-fix-c-build-state.md`) — the object-question do-support frame
/// (`svo::wh_what_object` + `svo::does_support` + `svo::bare_transitive_verb`)
/// over a quoted-mention NP, for concepts whose realization has no
/// grammatical bare-copula reading: an ADVERB takes neither a determiner
/// ("*what is a quickly") nor a bare predicative complement ("*what is
/// quickly" is not a well-formed definitional question) the way an adjective
/// does. Quoting the label licenses the mention reading a copula frame
/// cannot supply (Quine 1940:26 via SEP "Quotation" §3.1); "mean" asks
/// directly for the sense, exactly as `wh_definition`'s copula asks for the
/// referent.
///
/// The quote pair (U+201C/U+201D, LEFT/RIGHT DOUBLE QUOTATION MARK) is the
/// standard English orthographic double-quote convention — the same
/// `QuoteRole::Initial`/`Final` pair [`quote_glyphs`](super::super::lambek::quote_glyphs)
/// loads and [`collapse_quoted_spans`](super::super::lambek::tokenize) parses
/// back out, verified by this module's own round-trip test.
pub fn wh_mean(label: &str) -> String {
    format!("what does \u{201C}{label}\u{201D} mean")
}

/// A thin handle over the realization frames the capability generator turns
/// TYPED ARGS into interrogative surfaces with. Every surface is composed by the
/// grammar in THIS module (copula / determiner / wh-word / the polar inversion),
/// so the generator writes no interrogative text — it passes only labels in.
/// This is the realization ontology exposed as a frame-generator (the right
/// adjoint of parsing, this file's header): `Frames` owns all surfaces.
pub struct Frames;

impl Frames {
    /// The realization frame-generator, ready to realize probe questions.
    pub fn from_realization() -> Self {
        Frames
    }

    /// A wh-definition question for a concept label of a given part of speech —
    /// [`wh_definition`]. The POS drives the determiner (a noun takes an article,
    /// an adjective is realized bare), supplied by the caller from the concept.
    pub fn wh_definition(&self, label: &str, pos: LmfPos) -> String {
        wh_definition(label, pos)
    }

    /// A wh-mean question over a quoted-mention label — [`wh_mean`]. For a
    /// concept whose part of speech has no grammatical bare-copula
    /// definitional reading (an adverb: neither "*what is a quickly" nor
    /// "*what is quickly" is well-formed).
    pub fn wh_mean(&self, label: &str) -> String {
        wh_mean(label)
    }

    /// A polar is-a question over two labels with their parts of speech — the
    /// `sentence_question` inversion frame ("is a {subject} a {complement}?",
    /// the proven `(S[q]/NP)/NP + NP + NP → S[q]` realization) with each slot
    /// realized per-POS: nouns keep the article, a verb slot is its bare
    /// gerundial nominal ([`nominal_slot`]; "is coughing exhaling?").
    pub fn polar_isa(&self, subject: &str, spos: LmfPos, complement: &str, cpos: LmfPos) -> String {
        let s = nominal_slot(subject, spos);
        let c = nominal_slot(complement, cpos);
        let det_s = select_determiner_for_pos(&s, spos);
        let det_c = select_determiner_for_pos(&c, cpos);
        let cop = select_copula(&s);
        format!("{cop} {det_s}{s} {det_c}{c}?")
    }

    /// The surface a concept label takes in a nominal slot — [`nominal_slot`],
    /// exposed so the generator's EXPECTED properties are authored from the
    /// same realization the question used (one representation, the realizer's).
    pub fn nominal_slot(&self, label: &str, pos: LmfPos) -> String {
        nominal_slot(label, pos)
    }
}

// ---- Realization functions per frame ----

fn realize_assertion(content: &ResponseContent) -> String {
    // Self-model capability enumeration: "I can reason about: X, Y, Z." —
    // the LOADED component/domain list, generated, never a hardcoded list.
    if !content.capabilities.is_empty() {
        return format!("I can reason about: {}.", content.capabilities.join(", "));
    }

    if content.entities.len() >= 2 {
        let subject = &content.entities[0];
        let object = &content.entities[1];

        let is_taxonomy = content
            .predicate
            .as_ref()
            .is_some_and(|p| p == "is_a" || p == "is-a" || p == "isa");

        if is_taxonomy {
            // Generate: "Yes. {subject} is a {object}."
            let sentence = sentence_copula(subject, object);
            let mut result = format!("Yes. {sentence}.");
            for (entity, def) in &content.definitions {
                result.push_str(&format!("\n  {entity} -- {def}"));
            }
            return result;
        }

        let pred = content.predicate.as_deref().unwrap_or("relates to");
        let sentence = format!("{subject} {pred} {object}");
        return format!("Yes. {sentence}.");
    }

    if content.entities.len() == 1 {
        return realize_definition(&content.entities[0], &content.definitions);
    }

    content
        .predicate
        .as_deref()
        .unwrap_or("Understood.")
        .to_string()
}

/// Realize a negative assertion through the grammar. `connective` is the relation's
/// loaded surface ("part of") so a Parthood denial reads "is not part of"; `None`
/// is the is-a default ("is not a").
pub fn realize_negation(child: &str, parent: &str, connective: Option<&str>) -> String {
    let sentence = sentence_relation_neg(child, parent, connective);
    format!("No, {sentence}.")
}

/// Realize ONE entity's numbered gloss list — the block shared by
/// [`realize_definition`] (a single-entity `AssertKnowledge`/wh-question
/// answer, `content.entities.len() == 1`) and [`realize_comparison`] (a
/// `Comparison` answer, 2+ entities), so factoring this out for
/// `realize_comparison` leaves `realize_definition`'s own single-entity
/// surface form byte-identical to before.
fn realize_gloss_block(word: &str, definitions: &[&String]) -> String {
    if definitions.is_empty() {
        // Generate: "I know {word} but have no definition."
        // SVO: NP("I") + V("know") + NP(word) + conj("but") + V("have") + NP("no definition")
        return format!("{word}:\n  (no definitions available)",);
    }

    let mut lines = Vec::new();
    for (i, def) in definitions.iter().enumerate() {
        lines.push(format!("  {}. {def}", i + 1));
    }
    format!("{word}:\n{}", lines.join("\n"))
}

fn realize_definition(word: &str, definitions: &[(String, String)]) -> String {
    let defs: Vec<&String> = definitions.iter().map(|(_entity, def)| def).collect();
    realize_gloss_block(word, &defs)
}

/// Realize a [`ResponseFrame::Comparison`] — Mann & Thompson's (1988) RST
/// multinuclear Contrast relation, realized as one [`realize_gloss_block`]
/// PER named term, GROUPED by entity in `content.entities` order (never
/// just the first-listed term: Barker's (2011) derived relational noun
/// licenses EVERY overt participant, and this crate's own corpus fixture
/// sometimes names the SECOND-listed term as the answer key). Finally USES
/// `content.definitions`' per-definition entity field — unlike
/// `realize_definition`'s single-entity path, which never needed to (every
/// definition there already belongs to the one named entity).
fn realize_comparison(content: &ResponseContent) -> String {
    if content.entities.is_empty() {
        return "I do not have enough information to answer.".to_string();
    }
    let blocks: Vec<String> = content
        .entities
        .iter()
        .map(|entity| {
            let defs: Vec<&String> = content
                .definitions
                .iter()
                .filter(|(e, _)| e == entity)
                .map(|(_, def)| def)
                .collect();
            realize_gloss_block(entity, &defs)
        })
        .collect();
    blocks.join("\n\n")
}

fn realize_gap(content: &ResponseContent) -> String {
    if content.entities.len() == 1 {
        let word = &content.entities[0];
        // "I do not know the word {word}."
        return format!("I do not know the word \"{word}\".");
    }
    if content.entities.len() > 1 {
        let words: Vec<String> = content
            .entities
            .iter()
            .map(|w| format!("\"{w}\""))
            .collect();
        return format!("I do not know the words {}.", words.join(", "));
    }
    // No entities — general gap
    "I do not have enough information to answer.".to_string()
}

/// Realize an honest abstention over two KNOWN concepts whose relation is neither
/// derivable nor disprovable — the fifth epistemic cell (*vocabulary known,
/// proposition open*). Names BOTH concepts as known and the relation as
/// unconfirmed, so it never claims ignorance of a word it knows (Reiter 1978
/// closed-world). The discourse frame is a LOADED realization template
/// ([`abstain_frames::unproven_relation_template`]), Reiter & Dale (2000)
/// realization stage — NOT an inline literal here. This function only fills the
/// template's typed slots from the grammar in this module:
///   `{known}`  the "both X and Y" nominal (each concept with its determiner);
///   `{claim}`  the relation sentence over the pair ([`sentence_relation`], the
///              loaded connective from `content.predicate`, or the is-a default),
///              embedded under "cannot confirm" so it asserts nothing.
fn realize_abstain(content: &ResponseContent) -> String {
    use super::abstain_frames;
    // The template is loaded data. A missing row is a build defect (the committed
    // table always carries `unproven_relation`); fail closed to the vocabulary-gap
    // surface rather than fabricate a response literal here.
    let Some(template) = abstain_frames::unproven_relation_template() else {
        return realize_gap(content);
    };
    if content.entities.len() < 2 {
        // The UnprovenRelation branch guarantees two known concepts; a shorter
        // content is degenerate — abstain via the vocabulary-gap surface.
        return realize_gap(content);
    }
    let subject = &content.entities[0];
    let object = &content.entities[1];
    // The relation's LOADED surface ("part of"), threaded through the predicate
    // slot by the caller; `None` is the is-a copula default.
    let connective = content.predicate.as_deref();

    let known = format!(
        "{}{} and {}{}",
        select_determiner(subject),
        subject,
        select_determiner(object),
        object
    );
    let claim = sentence_relation(subject, object, connective);

    template
        .replace("{known}", &known)
        .replace("{claim}", &claim)
}

fn realize_suggestion(content: &ResponseContent) -> String {
    if content.entities.len() >= 2 {
        // Generate a question from the found concepts
        let question = sentence_question(&content.entities[0], &content.entities[1]);
        return format!(
            "I found {} concepts but could not parse the sentence.\nDid you mean: {question}",
            content.entities.len()
        );
    }
    if !content.entities.is_empty() {
        let words: Vec<String> = content
            .entities
            .iter()
            .map(|w| format!("\"{w}\""))
            .collect();
        return format!(
            "I know the words {} but could not understand the sentence structure.",
            words.join(", ")
        );
    }
    "I understood some of the input but could not form a complete interpretation.".to_string()
}

/// Realize an honest, entity-NAMING decline: the queried entity(ies) in
/// `content` ARE known (a resolved concept fed `answer_question`'s own
/// `AdmitLimitation` choice — e.g. a person-identifying "who" question, or a
/// rule-governed verdict "eligible for" nothing here can produce), but the
/// question's own illocutionary shape is not one this ontology can answer.
/// Mirrors [`realize_gap`]'s "name what I DO/DON'T know" honesty discipline
/// (Reiter 1978 closed-world assumption) — the difference from `realize_gap`
/// is exactly that split: `realize_gap` names an entity as UNKNOWN,
/// `realize_limitation` names one as KNOWN-but-not-answerable-this-way.
///
/// Before this fix `_content` was ignored entirely (a leading underscore, no
/// use in the body): every `AdmitLimitation` reply recited the SAME generic
/// "I do not understand" text plus a fixed "dog"/"mammal" worked example,
/// regardless of which real entity `answer_question` had actually resolved
/// and attached via [`ResponseContent::with_entity`] — a false "I know
/// nothing" impression on a turn that, in truth, knows exactly what was
/// asked about.
fn realize_limitation(content: &ResponseContent) -> String {
    if content.entities.is_empty() {
        // No entity resolved either — nothing concrete to name, so the
        // generic worked-example scaffold is the honest fallback (the
        // ORIGINAL, still-correct behavior for this degenerate case).
        let example = sentence_question("dog", "mammal");
        return format!("I do not understand. Try a question like: {example}");
    }
    let known: Vec<String> = content
        .entities
        .iter()
        .map(|e| format!("{}{}", select_determiner(e), e))
        .collect();
    let pronoun = if content.entities.len() > 1 {
        "them"
    } else {
        "it"
    };
    format!(
        "I know about {}, but I cannot answer that particular question about {}.",
        known.join(" and "),
        pronoun
    )
}

/// Realize the Malinowski (1923) "communion" route: a phatic exchange has no
/// propositional content to convey (info-content = ∅ BY DESIGN, not an
/// omission), so the reciprocal response is content-independent — the SAME
/// text regardless of which loaded interjection or `SpeechAct::Greeting`
/// triggered the frame, matching the phenomenon: what maintains the social
/// tie is the exchange itself, not any specific wording (Malinowski 1923,
/// *The Problem of Meaning in Primitive Languages*, in Ogden & Richards,
/// *The Meaning of Meaning*, Supplement I).
///
/// Honest scope statement, unlike [`realize_abstain`]'s loaded
/// `abstain_frames` template: this text is a fixed Rust literal, not read
/// from a loaded source. That is a deliberate difference, not an
/// unexamined shortcut — `abstain_frames` exists because abstention varies
/// by relation KIND (a real parametrization axis with more than one row);
/// a phatic return has no such axis by the cited theory's own claim (the
/// content is arbitrary and content-INDEPENDENT, so a "loaded template"
/// would be a table with exactly one row standing in for a single Rust
/// constant). The specific words are still grounded, not invented: "Hello"
/// is the canonical English greeting root the loaded `InterjectionKind::
/// Greeting` class itself names (`lexicon/pos.rs`), and the exchange
/// pattern (a return greeting, no propositional content) is Malinowski's
/// own description of the phenomenon, cited above.
fn realize_phatic_return() -> String {
    "Hello — I'm here.".to_string()
}

/// Realize a fully-known, fully-cited rule whose applicability for THIS
/// asker is blocked on a private fact never supplied (Bobrow, Kaplan,
/// Norman, Thompson & Winograd 1977, GUS: ask, don't guess). The discourse
/// frame is a LOADED realization template
/// ([`conditional_frames::rule_awaiting_fact_template`]), Reiter & Dale
/// (2000) realization stage — not an inline literal here. Fails closed to
/// [`realize_limitation`] on degenerate content (no presentation payload) or
/// a missing committed template row — the same fail-closed discipline
/// [`realize_abstain`] uses for its own loaded template.
fn realize_rule_awaiting_fact(content: &ResponseContent) -> String {
    use super::conditional_frames;
    let Some(p) = &content.conditional else {
        return realize_limitation(content);
    };
    let Some(template) = conditional_frames::rule_awaiting_fact_template() else {
        return realize_limitation(content);
    };
    template
        .replace("{rule_name}", &p.rule_name)
        .replace("{citation}", &p.citation)
        .replace("{rule_text}", &p.rule_text)
        .replace("{missing_facts}", &p.missing_facts.join(" and "))
}

fn realize_rule_applies(content: &ResponseContent) -> String {
    use super::conditional_frames;
    let Some(p) = &content.verdict else {
        return realize_limitation(content);
    };
    let Some(template) = conditional_frames::rule_applies_template() else {
        return realize_limitation(content);
    };
    template
        .replace("{rule_name}", &p.rule_name)
        .replace("{citation}", &p.citation)
        .replace("{rule_text}", &p.rule_text)
}

fn realize_rule_does_not_apply(content: &ResponseContent) -> String {
    use super::conditional_frames;
    let Some(p) = &content.verdict else {
        return realize_limitation(content);
    };
    let Some(template) = conditional_frames::rule_does_not_apply_template() else {
        return realize_limitation(content);
    };
    template
        .replace("{rule_name}", &p.rule_name)
        .replace("{citation}", &p.citation)
        .replace("{rule_text}", &p.rule_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn copula_sentence_generates() {
        assert_eq!(sentence_copula("dog", "mammal"), "a dog is a mammal");
        assert_eq!(sentence_copula("cat", "animal"), "a cat is an animal");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn copula_negation_generates() {
        assert_eq!(sentence_copula_neg("dog", "fish"), "a dog is not a fish");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn copula_question_generates() {
        assert_eq!(sentence_question("dog", "mammal"), "is a dog a mammal?");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn wh_definition_generates() {
        // Composed from the same copula + determiner selectors — the label is the
        // only content, never an authored question string. A noun takes the
        // phonological a/an article.
        assert_eq!(wh_definition("dog", LmfPos::Noun), "what is a dog");
        assert_eq!(wh_definition("animal", LmfPos::Noun), "what is an animal");
        // A predicate adjective is realized BARE — "*what is an able" would parse
        // `able` as a determined noun, never the predicate adjective the montague
        // extractor needs. The determiner is a nominal specifier; the adjective
        // takes none.
        assert_eq!(wh_definition("able", LmfPos::Adjective), "what is able");
        assert_eq!(wh_definition("red", LmfPos::Adjective), "what is red");
        // A satellite adjective (`s`) is an adjective too — also realized bare.
        assert_eq!(
            wh_definition("aback", LmfPos::SatelliteAdjective),
            "what is aback"
        );
        // A VERB definiendum is asked over its bare gerundial nominal (CGEL
        // pp. 1189/1220-1222; the -ing surface rides the dual-route
        // generation over the loaded AGID exceptions) — landed together with
        // its extraction path (the FIX-B law).
        assert_eq!(wh_definition("cough", LmfPos::Verb), "what is coughing");
        assert_eq!(
            wh_definition("hyperventilate", LmfPos::Verb),
            "what is hyperventilating"
        );
        assert_eq!(wh_definition("die", LmfPos::Verb), "what is dying");
        // A proper-noun label drops the determiner (the `Frames` handle threads
        // the identical realizer).
        assert_eq!(
            Frames::from_realization().wh_definition("Alice", LmfPos::Noun),
            "what is Alice"
        );
        // The polar frame realizes each slot per-POS: nouns keep the article,
        // verb slots are bare gerundial nominals.
        assert_eq!(
            Frames::from_realization().polar_isa("dog", LmfPos::Noun, "mammal", LmfPos::Noun),
            "is a dog a mammal?"
        );
        assert_eq!(
            Frames::from_realization().polar_isa("cough", LmfPos::Verb, "exhale", LmfPos::Verb),
            "is coughing exhaling?"
        );
        assert_eq!(
            Frames::from_realization().nominal_slot("exhale", LmfPos::Verb),
            "exhaling"
        );
        // A PHRASAL verb is NOT gerundized — whole-phrase suffixation
        // ("care foring") is garbage, and the head-inflected surface
        // ("caring for") is not in the multi-word surface index; the
        // collapsed-nominal frame the multi-word slice measured green stays.
        assert_eq!(
            Frames::from_realization().nominal_slot("care for", LmfPos::Verb),
            "care for"
        );
        assert_eq!(
            wh_definition("care for", LmfPos::Verb),
            "what is a care for"
        );
        assert_eq!(
            Frames::from_realization().polar_isa("cough", LmfPos::Verb, "cough out", LmfPos::Verb),
            "is coughing a cough out?"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn wh_mean_quotes_the_label() {
        assert_eq!(wh_mean("quickly"), "what does \u{201C}quickly\u{201D} mean");
        assert_eq!(
            Frames::from_realization().wh_mean("deadly"),
            "what does \u{201C}deadly\u{201D} mean"
        );
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn wh_mean_round_trips_through_the_tokenizer_quote_collapse() {
        // Grounds the realizer's chosen glyph pair against the tokenizer's
        // loaded quote-glyph vocabulary directly — realize.rs and
        // tokenize.rs must never drift apart on which glyphs mark a
        // quoted-mention span.
        use crate::cognitive::linguistics::lambek::quote_glyphs;
        use crate::cognitive::linguistics::lambek::tokenize::collapse_quoted_spans;
        let vocab = quote_glyphs::load();
        let question = wh_mean("quickly");
        let words: Vec<String> = question.split_whitespace().map(str::to_string).collect();
        let sentence_initial = vec![false; words.len()];
        let (collapsed, quoted, _sentence_initial) =
            collapse_quoted_spans(words, sentence_initial, &vocab);
        assert_eq!(
            collapsed.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["what", "does", "quickly", "mean"]
        );
        assert_eq!(quoted, vec![false, false, true, false]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn determiner_selection() {
        assert_eq!(select_determiner("dog"), "a ");
        assert_eq!(select_determiner("animal"), "an ");
        assert_eq!(select_determiner("Alice"), ""); // proper noun
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn assert_taxonomy_uses_grammar() {
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_predicate("is_a")
            .with_entity("dog")
            .with_entity("mammal")
            .with_definition("dog", "a domesticated canine")
            .with_definition("mammal", "warm-blooded vertebrate");

        let text = realize(&content);
        assert!(text.starts_with("Yes. a dog is a mammal."));
        assert!(text.contains("dog -- a domesticated canine"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn capability_enumeration_uses_grammar() {
        // A self-model capability query realizes the LOADED capability list
        // as generated text, never a hardcoded sentence -- the list content
        // is entirely caller-supplied data.
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_capabilities(vec!["English".into(), "USCTitle42".into()]);
        let text = realize(&content);
        assert_eq!(text, "I can reason about: English, USCTitle42.");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn phatic_return_realizes_a_reciprocal_response_content_independent() {
        // Malinowski's "communion" property: the realized text does not
        // depend on any caller-supplied content (there is none to depend
        // on) -- two independently built PhaticReturn contents realize
        // identically, and the text is never the AdmitLimitation fallback.
        let a = ResponseContent::new(ResponseFrame::PhaticReturn);
        let b = ResponseContent::new(ResponseFrame::PhaticReturn).with_entity("irrelevant");
        let text_a = realize(&a);
        let text_b = realize(&b);
        assert_eq!(text_a, text_b);
        assert!(!text_a.is_empty());
        assert!(!text_a.contains("I do not understand"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn empty_capability_list_falls_through_to_the_ordinary_assertion_path() {
        // No capabilities attached -> the ordinary (non-capability) assertion
        // realization runs, never a bare "I can reason about: ." sentence.
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge);
        assert!(!realize(&content).contains("I can reason about"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn negation_uses_grammar() {
        // is-a negation (no connective) is unchanged.
        assert_eq!(
            realize_negation("dog", "fish", None),
            "No, a dog is not a fish."
        );
        // A relational denial reads "is not <connective>".
        assert_eq!(
            realize_negation("section", "subsection", Some("part of")),
            "No, a section is not part of a subsection."
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn relation_sentence_uses_the_loaded_connective() {
        // None → the is-a copula; Some(surface) → "is <surface>".
        assert_eq!(
            sentence_relation("dog", "mammal", None),
            "a dog is a mammal"
        );
        assert_eq!(
            sentence_relation("subsection", "section", Some("part of")),
            "a subsection is part of a section"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn definition_format() {
        let content = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_entity("dog")
            .with_definition("dog", "a domesticated canine");

        let text = realize(&content);
        assert!(text.contains("dog:"));
        assert!(text.contains("a domesticated canine"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gap_single_word() {
        let content = ResponseContent::new(ResponseFrame::UnknownVocabulary).with_entity("xyzzy");
        let text = realize(&content);
        assert_eq!(text, "I do not know the word \"xyzzy\".");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gap_multiple_words() {
        let content = ResponseContent::new(ResponseFrame::UnknownVocabulary)
            .with_entity("foo")
            .with_entity("bar");
        let text = realize(&content);
        assert!(text.contains("\"foo\""));
        assert!(text.contains("\"bar\""));
    }

    /// The honest-abstain surface over two KNOWN concepts: it NAMES both as known
    /// and the relation as unconfirmed, and never reuses the vocabulary-gap
    /// "I do not know the word" surface. The discourse frame is loaded data
    /// (`abstain-frames.prx`); this test asserts the FILLED slots, not the frame.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn abstain_names_both_concepts_as_known() {
        let content = ResponseContent::new(ResponseFrame::UnprovenRelation)
            .with_entity("cat")
            .with_entity("dog");
        let text = realize(&content);
        // Both concepts are named (the honest-abstention doctrine at the surface).
        assert!(text.contains("a cat"), "names the subject as known: {text}");
        assert!(text.contains("a dog"), "names the object as known: {text}");
        // It must NOT masquerade as a vocabulary gap over the known words.
        assert!(
            !text.contains("I do not know the word"),
            "an unproven-relation abstain must not claim vocabulary ignorance: {text}"
        );
        // The is-a claim is embedded (asserts nothing under "cannot confirm").
        assert!(
            text.contains("a cat is a dog"),
            "embeds the unconfirmed claim: {text}"
        );
    }

    /// A relational (non-is-a) abstain threads the LOADED connective through the
    /// predicate slot, so the embedded claim reads "is part of", not "is a".
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn abstain_uses_the_loaded_relation_connective() {
        let content = ResponseContent::new(ResponseFrame::UnprovenRelation)
            .with_predicate("part of")
            .with_entity("section")
            .with_entity("clause");
        let text = realize(&content);
        assert!(
            text.contains("a section is part of a clause"),
            "embeds the relational claim via the loaded connective: {text}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn suggestion_generates_question() {
        let content = ResponseContent::new(ResponseFrame::SuggestInterpretation)
            .with_entity("dog")
            .with_entity("mammal");
        let text = realize(&content);
        assert!(text.contains("is a dog a mammal?"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn limitation_generates_example() {
        let content = ResponseContent::new(ResponseFrame::AdmitLimitation);
        let text = realize(&content);
        assert!(text.contains("is a dog a mammal?"));
    }

    /// Regression guard for a real, reported defect: `realize_limitation`
    /// used to ignore its own `content` parameter entirely, so an
    /// `AdmitLimitation` reply over a KNOWN, resolved entity (e.g.
    /// `answer_question`'s "who is eligible for X" branch, which attaches
    /// the resolved entity via `with_entity` before declining) still
    /// recited the same content-free "I do not understand" text — a false
    /// "I know nothing" impression. Once `content` carries an entity, the
    /// realized text must NAME it (not the generic dog/mammal scaffold) and
    /// must not repeat the "I do not understand" framing, which falsely
    /// claims total ignorance over a turn that in truth knows the entity.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn limitation_with_a_known_entity_names_it_instead_of_the_generic_example() {
        let content = ResponseContent::new(ResponseFrame::AdmitLimitation).with_entity("DCH");
        let text = realize(&content);
        assert!(
            text.contains("DCH"),
            "a known entity must be named, not silently dropped: {text}"
        );
        assert!(
            !text.contains("is a dog a mammal?"),
            "a known-entity decline must not fall back to the content-free \
             worked example: {text}"
        );
        assert!(
            !text.starts_with("I do not understand"),
            "a known entity is not \"not understood\" — the honest framing is \
             \"known but can't answer that way\": {text}"
        );
    }

    /// The multi-entity case pluralizes its closing pronoun ("them", not
    /// "it") and names every entity — the same "never drop a known entity"
    /// discipline `realize_abstain`'s two-entity naming already enforces.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn limitation_with_two_known_entities_names_both_and_pluralizes() {
        let content = ResponseContent::new(ResponseFrame::AdmitLimitation)
            .with_entity("cat")
            .with_entity("dog");
        let text = realize(&content);
        assert!(text.contains("a cat"), "names the first entity: {text}");
        assert!(text.contains("a dog"), "names the second entity: {text}");
        assert!(
            text.contains("them"),
            "two known entities pluralize the closing pronoun: {text}"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn frame_determines_structure() {
        let base = ResponseContent::new(ResponseFrame::AssertKnowledge)
            .with_entity("dog")
            .with_entity("animal")
            .with_predicate("is_a");

        let assert_text = realize(&base);
        assert!(assert_text.starts_with("Yes."));

        let mut suggest = base.clone();
        suggest.frame = ResponseFrame::SuggestInterpretation;
        let suggest_text = realize(&suggest);
        assert!(suggest_text.contains("Did you mean"));
    }

    // === The "Why?" layer (realize_why) ===
    //
    // Each explanation is a loaded frame (explain-frames.prx) filled from the
    // ACTUAL outcome data of a turn; these tests assert the filled slots and the
    // provenance split, never author the frame in code.

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn why_grounded_answer_names_the_projected_source() {
        // An answer that reasoned over a loaded source names it by its plain
        // label — never its raw internal OntologyName.
        let sources = [OntologyName::new("caregiving_lexicon")];
        let why = realize_why(ResponseFrame::AssertKnowledge, true, &sources, &[])
            .expect("a grounded answer has a why");
        assert!(
            why.contains("the Family Caregiving lexicon"),
            "names the projected source: {why}"
        );
        assert!(
            !why.contains("caregiving_lexicon"),
            "the raw OntologyName must never leak: {why}"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn why_ungrounded_answer_states_the_built_in_substrate() {
        // Empty reasoned_over is the honest "answered from the compiled substrate"
        // case — it must NOT claim a document was loaded.
        let why = realize_why(ResponseFrame::AssertKnowledge, true, &[], &[])
            .expect("an ungrounded answer has a why");
        assert!(
            why.contains("built-in"),
            "states the built-in substrate: {why}"
        );
        assert!(
            !why.contains("loaded documents"),
            "an ungrounded answer must not claim loaded documents: {why}"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn why_vocabulary_gap_names_the_unresolved_terms() {
        let unresolved = vec!["florb".to_string()];
        let why = realize_why(ResponseFrame::UnknownVocabulary, false, &[], &unresolved)
            .expect("a vocabulary-gap abstention has a why");
        assert!(why.contains("florb"), "names the unresolved term: {why}");
        // A term-less vocabulary gap still explains, without an empty slot.
        let untermed = realize_why(ResponseFrame::UnknownVocabulary, false, &[], &[])
            .expect("a term-less vocabulary gap still has a why");
        assert!(
            !untermed.contains("{terms}"),
            "the slot is filled or dropped, never left raw: {untermed}"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn why_self_explaining_frames_get_no_panel() {
        // The frames whose primary response already carries its reason produce no
        // Why? — a second sentence would only repeat the answer.
        for frame in [
            ResponseFrame::UnprovenRelation,
            ResponseFrame::RuleAwaitingFact,
            ResponseFrame::RuleApplies,
            ResponseFrame::RuleDoesNotApply,
            ResponseFrame::Comparison,
            ResponseFrame::PhaticReturn,
        ] {
            assert_eq!(
                realize_why(frame, false, &[], &[]),
                None,
                "{frame:?} self-explains — no why"
            );
        }
        // A degenerate AssertKnowledge that did not ground an answer: no honest
        // provenance to state, so no why either.
        assert_eq!(
            realize_why(ResponseFrame::AssertKnowledge, false, &[], &[]),
            None
        );
    }

    /// Critique fix (d): rendered why-text NEVER contains an un-projected
    /// snake_case OntologyName. A mix of a named built-in and an unregistered
    /// runtime source both project cleanly — no `foo_bar` identifier survives.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn why_never_leaks_a_snake_case_identifier() {
        let sources = [
            OntologyName::new("usc_title_42"),
            OntologyName::new("some_unregistered_runtime_corpus"),
        ];
        let why =
            realize_why(ResponseFrame::AssertKnowledge, true, &sources, &[]).expect("grounded why");
        assert!(
            !why.contains("usc_title_42"),
            "raw title identifier leaked: {why}"
        );
        assert!(
            !why.contains("some_unregistered_runtime_corpus"),
            "raw runtime identifier leaked: {why}"
        );
        assert!(
            why.contains("Title 42"),
            "the named source is projected: {why}"
        );
        assert!(
            why.contains("your loaded documents"),
            "the unnamed runtime source projects to the generic fallback: {why}"
        );
    }

    /// The three-way provenance split (critique fix c): EMPTY → the built-in
    /// substrate; a NAMED built-in → its label; an UNNAMED runtime source → the
    /// generic fallback — three distinct, honest surfaces.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn why_provenance_split_is_three_way() {
        let empty = realize_why(ResponseFrame::AssertKnowledge, true, &[], &[]).expect("empty why");
        let named = realize_why(
            ResponseFrame::AssertKnowledge,
            true,
            &[OntologyName::new("hcbs_compliance_lexicon")],
            &[],
        )
        .expect("named why");
        let unnamed = realize_why(
            ResponseFrame::AssertKnowledge,
            true,
            &[OntologyName::new("mystery_corpus_2027")],
            &[],
        )
        .expect("unnamed why");
        assert!(empty.contains("built-in"), "{empty}");
        assert!(named.contains("HCBS"), "{named}");
        assert!(unnamed.contains("your loaded documents"), "{unnamed}");
        // The three are genuinely distinct surfaces.
        assert_ne!(empty, named);
        assert_ne!(named, unnamed);
        assert_ne!(empty, unnamed);
    }
}
