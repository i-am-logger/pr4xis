//! The CCG supertag/rule cost table, carried as LOADED data and interpreted —
//! derivation ranking done the praxis way (weights-as-data), NOT a Rust
//! `match` of numbers.
//!
//! This is the cited, buildable floor of statistical CCG derivation ranking:
//! per-category costs are the negative logs of PUBLISHED CCGbank frequencies
//! (Hockenmaier 2003, PhD thesis, Table 5.5 p. 176; Hockenmaier & Steedman
//! 2005, CCGbank User's Manual, Table 4.3 p. 86; Hockenmaier & Steedman 2007,
//! Computational Linguistics 33(3) §8), read off the published tables — not
//! invented weights, not trained parameters (the full Clark & Curran 2007
//! log-linear model is a separate research effort). The chart
//! ([`chart_reduce`](super::reduce::chart_reduce)) keeps a preferred
//! derivation per (span, type) — the weighted-CYK shape of Goodman (1999)
//! "Semiring Parsing" (his Viterbi semiring; the (min, +) formulation is the
//! broader literature's tropical semiring), with cost as the final component
//! of the preference key.
//!
//! Load path mirrors the OLiA→CCG projection
//! ([`category_projection`](super::category_projection)): the committed
//! content-addressed `.prx` is `include_bytes!`-embedded, decoded through the
//! generalized raw-source gate, parsed by ONE generic row loop, and cached in
//! a process `OnceLock` (`std`) or rebuilt by value (`no_std`). Every category
//! notation must parse ([`parse_category`]) and every cost must be finite and
//! positive — build invariants, asserted fail-closed at load.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::notation_parser::parse_category;
use super::types::LambekType;
use crate::formal::math::quantity::unit::{NAT, UNITLESS};
use crate::formal::math::quantity::value::Quantity;

/// The committed supertag-cost table `.prx` — the content-addressed envelope
/// carrying the cited CCGbank frequency rows. The raw `.tsv` is authored
/// source-of-truth (git-tracked, EXCLUDED from the published crate); only this
/// `.prx` is committed + embedded and ships. Loaded through the generalized
/// raw-source gate, feature-independent so the table builds on default,
/// `no_std` and wasm.
const SUPERTAG_COSTS_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/grammar/ccg-supertag-costs.prx"
));

/// The registry `name@version` key the committed `.prx` is pinned under (see
/// `[sources.ccg_supertag_costs]` in `praxis.toml`).
const NAME: &str = "ccg_supertag_costs";
const VERSION: &str = "2026";

/// A derivation cost in nats — the negative natural log of a loaded CCGbank
/// relative frequency ([`NAT`], ISO/IEC 80000-13:2008 unit entry 13-24.c;
/// Shannon 1948, Introduction: "natural units").
///
/// The cost scalar of the weighted chart (Goodman 1999, Semiring Parsing —
/// the (min, +)/Viterbi shape): [`DerivationCost::plus`] is the ⊗
/// (probability product = cost sum); the chart's relaxation prefers lower
/// cost within its lexicographic key. Constructed ONLY from a loaded
/// [`Quantity`] in [`NAT`] (the one codec crossing) or as
/// [`DerivationCost::ZERO`] (the ⊗-identity: probability 1). Totally ordered
/// via `total_cmp` — loaded values are asserted finite at load, so the total
/// order is the numeric one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DerivationCost(f64);

impl DerivationCost {
    /// The ⊗-identity: cost 0 nats = probability 1 (a step that consumes no
    /// probability mass — binary application in the unigram model).
    pub const ZERO: Self = Self(0.0);

    /// The one crossing from the loaded, typed world: a [`Quantity`] carrying
    /// nats becomes a chart-side cost. `None` if the quantity is not in the
    /// information dimension or not finite (fail-closed).
    pub fn from_quantity(q: &Quantity) -> Option<Self> {
        (q.dimension == NAT.dimension && q.value.is_finite()).then_some(Self(q.value))
    }

    /// Semiring ⊗: costs add (probabilities multiply).
    pub fn plus(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Eq for DerivationCost {}

impl PartialOrd for DerivationCost {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DerivationCost {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// A loaded unary (type-changing) rule row: `from → to` at `cost`.
/// CCGbank's type-changing rules (Hockenmaier & Steedman 2007 §4.6; the
/// User's Manual §2.5.5 describes the bare-noun `NP → N` as "the simplest and
/// most common one", frequency in its Table 4.3) as chart rows. The cost is the
/// negative log of the rule's published corpus frequency; it is asserted
/// strictly positive at load, which is what makes the chart's unary closure a
/// terminating fixpoint and makes the un-raised derivation win wherever both
/// derivations share their leaves.
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryRule {
    pub from: LambekType,
    pub to: LambekType,
    pub cost: DerivationCost,
}

/// The loaded cost table: lexical unigram costs keyed by category, the unary
/// rule rows, and the default (hapax) cost for categories the published table
/// does not list (CL 33(3) §8.2 p. 389: 439 categories occur exactly once, so
/// an unlisted category is priced as one of them).
#[derive(Debug, Clone)]
pub struct SupertagCostTable {
    lexical: hashbrown::HashMap<LambekType, DerivationCost>,
    unary: Vec<UnaryRule>,
    default_cost: DerivationCost,
}

impl SupertagCostTable {
    /// The cost of assigning `category` to a leaf: its loaded unigram cost, or
    /// the hapax default for a category the published table does not list.
    pub fn lexical_cost(&self, category: &LambekType) -> DerivationCost {
        self.lexical
            .get(category)
            .copied()
            .unwrap_or(self.default_cost)
    }

    /// The loaded unary (type-changing) rule rows.
    pub fn unary_rules(&self) -> &[UnaryRule] {
        &self.unary
    }

    /// How many lexical rows the published table contributed.
    pub fn lexical_len(&self) -> Quantity {
        Quantity::from_unit(self.lexical.len() as f64, &UNITLESS)
    }

    /// The hapax default cost (the price of an unlisted category).
    pub fn default_cost(&self) -> DerivationCost {
        self.default_cost
    }

    /// A NEW table carrying every lexical row and default cost of `self`,
    /// PLUS `extra` unary rules — never mutating `self` and never touching
    /// the SHARED production table [`supertag_cost_table`]. Composition,
    /// not duplication: the ~13+ published lexical rows are reused by
    /// reference-then-clone rather than re-parsed from a second copy of the
    /// TSV.
    ///
    /// Exists for exactly one caller,
    /// [`crate::social::judicial::statute_structure::grounding::defines_pointers`]:
    /// a bare-noun-phrase unary rule ([`bare_noun_phrase_unary_rule`]) is
    /// licensed within a statutory "means"/"includes" definiens (a register
    /// that routinely uses an unmodified common-noun definiens — Dickerson
    /// 1986) but is DELIBERATELY ABSENT from the shared production table a
    /// corpus-gate measurement already rejected at the GLOBAL level (see this
    /// module's own `tests::the_shipped_table_carries_no_unary_rows` test:
    /// define −6, 2026-07-10). This method is how a SCOPED caller can still use the
    /// published CCGbank statistic without reopening that measured,
    /// deliberate decision.
    pub fn with_extra_unary(&self, extra: Vec<UnaryRule>) -> Self {
        let mut unary = self.unary.clone();
        unary.extend(extra);
        Self {
            lexical: self.lexical.clone(),
            unary,
            default_cost: self.default_cost,
        }
    }
}

/// The bare-noun-phrase unary rule — CCGbank's own `NP → N` (our `N → NP`,
/// the SAME direction [`crate::cognitive::linguistics::lambek::reduce`]'s
/// own `fixture_with_bare_np_rule` test fixture already carries): a
/// determiner-less common noun promoted directly to a saturated noun
/// phrase. Hockenmaier & Steedman (2005), *CCGbank User's Manual*
/// MS-CIS-05-09, University of Pennsylvania, §2.5.5 ("the simplest and
/// most common" unary type-changing rule) + Table 4.3 p. 86 (count
/// 115,516 / total 929,552) — the SAME published statistic already cited
/// in this module's own [`UnaryRule`] doc and reused (not re-invented)
/// here for a scoped production use (see [`SupertagCostTable::with_extra_unary`]
/// for why it is scoped rather than loaded into the shared table).
pub fn bare_noun_phrase_unary_rule() -> UnaryRule {
    let ratio = Quantity::from_unit(115_516.0_f64 / 929_552.0_f64, &UNITLESS);
    let cost = Quantity::from_unit(-ratio.value.ln(), &NAT);
    UnaryRule {
        from: LambekType::n(),
        to: LambekType::np(),
        cost: DerivationCost::from_quantity(&cost)
            .expect("the published NP -> N ratio is a finite positive nat"),
    }
}

/// CCGbank's REDUCED-PASSIVE-RELATIVE type-changing rule — `S[pss]\NP →
/// NP\NP` in CCGbank notation, `NP\S[pss] → NP\NP` in this grammar's.
/// Hockenmaier & Steedman (2005), *CCGbank User's Manual*, MS-CIS-05-09,
/// University of Pennsylvania, §3.8 p. 55, schema (53) `S$ ⇒ X|X` and its
/// FIRST listed instantiation (54)a:
///
/// > (54) a. `S[pss]\NP ⇒ NP\NP`  "workers [exposed to it]"
///
/// with the worked derivation in Figure 3.1 (same page): "A form of
/// asbestos [once used to make cigarette filters]" reduces `S[pss]\NP` to
/// `NP\NP` and applies it to the head NP. §3.8's own rationale is exactly
/// why this is a GRAMMAR rule and not a second lexical entry: without it, "a
/// past participle such as *used* receives different categories depending on
/// whether it occurs in a reduced relative or a main verb phrase", and every
/// modifier of it multiplies with that choice.
///
/// # Cost
///
/// PRICED AS A HAPAX, the same fail-closed convention the loaded table's own
/// `default` row already applies to an unlisted LEXICAL category. This rule
/// is NOT among the 20 rule instantiations Table 4.3 p. 86 publishes (whose
/// smallest is 8,184), so its exact count is unpublished; Table 4.4 (same
/// page) records that 1,146 of the grammar's 3,262 rule types occur exactly
/// once in sections 02-21, so count 1 is a real, published population for an
/// unlisted rule — and its `-ln(1/929,552)` price is therefore an UPPER
/// bound on the rule's true cost (929,552 = the sections 02-21 token count,
/// Hockenmaier & Steedman 2007, *Computational Linguistics* 33(3) §8 p. 387
/// — the SAME denominator [`bare_noun_phrase_unary_rule`] uses, so the two
/// scoped rows are commensurable).
///
/// Over-pricing is the SAFE direction and is deliberate: a hapax-priced
/// type change can never displace a derivation that parses without it, so
/// the participial reading fires only where nothing else parses at all —
/// the identical safety property [`DeterminerFrameNeedsNoTypeChanging`]
/// asserts for the bare-NP row.
///
/// Scoped exactly like [`bare_noun_phrase_unary_rule`]: composed onto a NEW
/// table via [`SupertagCostTable::with_extra_unary`] by
/// `statute_structure::grounding::definiens_cost_table`, NEVER loaded into
/// the shared production table the live chat pipeline runs on.
pub fn reduced_passive_relative_unary_rule() -> UnaryRule {
    let ratio = Quantity::from_unit(1.0_f64 / 929_552.0_f64, &UNITLESS);
    let cost = Quantity::from_unit(-ratio.value.ln(), &NAT);
    UnaryRule {
        from: super::types::svo::passive_participle_verb(),
        to: super::types::svo::reduced_relative_postmodifier(),
        cost: DerivationCost::from_quantity(&cost)
            .expect("the published hapax rule ratio is a finite positive nat"),
    }
}

/// Lower a loaded TSV cost cell to a [`DerivationCost`] through the typed
/// [`Quantity`] crossing — the value is nats by the table's column contract.
/// Fail-closed on non-finite or non-positive values (a negative log of a
/// published `count < total` ratio is strictly positive).
fn lower_cost(raw: &str, row: &str) -> DerivationCost {
    let value: f64 = raw
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("cost cell {raw:?} does not parse as a number in row {row:?}"));
    let q = Quantity::from_unit(value, &NAT);
    let cost = DerivationCost::from_quantity(&q)
        .unwrap_or_else(|| panic!("cost cell {raw:?} is not a finite nat quantity in row {row:?}"));
    assert!(
        cost > DerivationCost::ZERO,
        "cost cell {raw:?} must be strictly positive (a negative log of count < total) in row {row:?}"
    );
    cost
}

/// Parse a repo-notation category cell (build invariant: every row parses).
fn lower_category(raw: &str, row: &str) -> LambekType {
    parse_category(raw.trim())
        .unwrap_or_else(|| panic!("category {raw:?} does not parse in row {row:?}"))
}

/// Decode the committed envelope and parse it — the loaded table.
pub fn build_table() -> SupertagCostTable {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, SUPERTAG_COSTS_PRX);
    parse_table(&tsv)
}

/// Parse a cost-table TSV into the cost table — the ONE generic interpreter
/// (never a per-category arm). Rows are kind-discriminated: `lex`, `unary`,
/// `default`. Public so the chart engine's tests can drive
/// [`chart_reduce_with_costs`](super::reduce::chart_reduce_with_costs) with a
/// fixture parsed through this same loop.
pub fn parse_table(tsv: &str) -> SupertagCostTable {
    let mut lexical: hashbrown::HashMap<LambekType, DerivationCost> = hashbrown::HashMap::new();
    let mut unary: Vec<UnaryRule> = Vec::new();
    let mut default_cost: Option<DerivationCost> = None;
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        match cols.as_slice() {
            // lex <repo-category> <ccgbank-category> <count> <total> <cost>
            ["lex", category, _ccgbank, _count, _total, cost] => {
                let key = lower_category(category, line);
                let prior = lexical.insert(key, lower_cost(cost, line));
                assert!(prior.is_none(), "duplicate lex row for {category:?}");
            }
            // unary <repo-from> <repo-to> <ccgbank-rule> <count> <total> <cost>
            ["unary", from, to, _ccgbank, _count, _total, cost] => {
                let from = lower_category(from, line);
                let to = lower_category(to, line);
                assert!(from != to, "a unary row must change the category: {line:?}");
                unary.push(UnaryRule {
                    from,
                    to,
                    cost: lower_cost(cost, line),
                });
            }
            // default <count> <total> <cost>
            ["default", _count, _total, cost] => {
                assert!(default_cost.is_none(), "duplicate default row");
                default_cost = Some(lower_cost(cost, line));
            }
            _ => panic!("unrecognized supertag-cost row shape: {line:?}"),
        }
    }
    let default_cost = default_cost.expect("the committed table carries the hapax default row");
    assert!(!lexical.is_empty(), "the committed table carries lex rows");
    // The default prices an UNLISTED category; a published row must never cost
    // more than the hapax floor (count ≥ 1 ⇒ cost ≤ -ln(1/total)).
    for (category, cost) in &lexical {
        assert!(
            *cost <= default_cost,
            "lex row {} costs more than the hapax default",
            category.notation()
        );
    }
    SupertagCostTable {
        lexical,
        unary,
        default_cost,
    }
}

/// The loaded supertag-cost table, cached for the process (`std`).
#[cfg(feature = "std")]
pub fn supertag_cost_table() -> &'static SupertagCostTable {
    use std::sync::OnceLock;
    static TABLE: OnceLock<SupertagCostTable> = OnceLock::new();
    TABLE.get_or_init(build_table)
}

/// Axiom: the determiner-framed polar question — the corpus's canonical
/// `is a ⟨N⟩ a ⟨N⟩` shape — is carried by the DIRECT (un-raised) derivation:
/// its min-cost parse under the LOADED table reaches `S[q]` with ZERO
/// type-changing steps.
///
/// This is the safety property whose absence made a naive (unranked) `N → NP`
/// chart rule regress the corpus (+273 is-a / +274 directional-No): in a
/// boolean chart the raised reading wins by iteration order; in the Viterbi
/// chart the type-changing row consumes probability mass (its published
/// CCGbank cost), so wherever the determiner path exists it is strictly
/// cheaper, and the marked rule fires only where nothing else parses
/// (determiner-less arguments — exactly where it is correct).
pub struct DeterminerFrameNeedsNoTypeChanging;

impl pr4xis::ontology::Axiom for DeterminerFrameNeedsNoTypeChanging {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        use super::reduce::chart_reduce;
        use super::types::svo;

        // The tokenizer's canonical assignment for `is a dog a mammal` —
        // position-0 copula → question_copula (+ its predicative alternative),
        // determiners, WordNet nouns (pinned by the tokenize/projection
        // oracles; svo constructors ARE those loaded rows' values).
        let words: Vec<String> = ["is", "a", "dog", "a", "mammal"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        let type_sets: Vec<Vec<LambekType>> = vec![
            vec![svo::question_copula(), svo::question_copula_pred()],
            vec![svo::determiner()],
            vec![svo::noun()],
            vec![svo::determiner()],
            vec![svo::noun()],
        ];
        let result = chart_reduce(&words, &type_sets);
        let q_goal = result.final_type == Some(LambekType::q());
        if result.success && q_goal && result.unary_steps == 0 {
            Ok(alloc::boxed::Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(alloc::boxed::Box::new(SimpleCounterexample::new(
                self.meta(),
            )))
        }
    }

    pr4xis::axiom_meta!(
        "DeterminerFrameNeedsNoTypeChanging",
        "the min-cost derivation of the determiner-framed polar question (`is a N a N` → S[q]) under the loaded CCGbank cost table uses zero unary type-changing steps — the raised reading never displaces the direct one where the determiner path exists, because the type-changing row consumes its published probability mass",
        "Hockenmaier & Steedman (2007) CCGbank, Computational Linguistics 33(3) §4.6 (type-changing rules); Hockenmaier & Steedman (2005) CCGbank User's Manual MS-CIS-05-09 §2.5.5 (the bare-noun NP → N rule) + Table 4.3 p. 86 (its frequency); Eisner (1996) Efficient Normal-Form Parsing for CCG, ACL (unranked competing derivations obscure true ambiguity); Goodman (1999) Semiring Parsing (Viterbi selection)"
    );
}
pr4xis::register_axiom!(DeterminerFrameNeedsNoTypeChanging, constructor);

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::lambek::types::svo;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_table_loads_and_prices_the_svo_categories() {
        let table = supertag_cost_table();
        // The published rows price the core svo categories BELOW the hapax
        // default, and the ranking follows the published counts: the common
        // noun (10,498) is the cheapest category in the table.
        let n = table.lexical_cost(&svo::noun());
        let np = table.lexical_cost(&svo::proper_noun());
        let det = table.lexical_cost(&svo::determiner());
        let adj = table.lexical_cost(&svo::adjective());
        assert!(n < adj && adj < det && det < np, "N < N/N < NP/N < NP");
        assert!(np < table.default_cost());
        // The wh-question category (count 8) is the rarest published row —
        // dearer than the bare-NP reading of the same word. This is exactly
        // why the goal keeps the featured-S preference PRIMARY and cost
        // secondary: cost alone would demote every wh-question.
        let wh = table.lexical_cost(&svo::wh_what());
        assert!(wh > np && wh < table.default_cost());
        // A category the published table does not list is priced as a hapax.
        let unseen = table.lexical_cost(&svo::question_copula());
        assert_eq!(unseen, table.default_cost());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_cost_column_is_the_negative_log_of_the_published_ratio() {
        // The TSV's cost column is DERIVED from its published (count, total)
        // columns; recompute -ln(count/total) here (std has `ln`) and hold the
        // committed column to it. Tolerance is half an ulp of the 6-decimal
        // authored precision.
        use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
        let tsv = raw_source_text_embedded(NAME, VERSION, SUPERTAG_COSTS_PRX);
        let mut checked = 0usize;
        for line in tsv.lines() {
            let line = line.trim_end();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            let (count, total, cost) = match cols.as_slice() {
                ["lex", _, _, count, total, cost] => (count, total, cost),
                ["unary", _, _, _, count, total, cost] => (count, total, cost),
                ["default", count, total, cost] => (count, total, cost),
                _ => panic!("unrecognized row shape: {line:?}"),
            };
            let count: f64 = count.parse().expect("published count");
            let total: f64 = total.parse().expect("published total");
            let cost: f64 = cost.parse().expect("derived cost");
            let recomputed = -(count / total).ln();
            assert!(
                (cost - recomputed).abs() < 5e-7,
                "cost column {cost} drifts from -ln({count}/{total}) = {recomputed} in {line:?}"
            );
            checked += 1;
        }
        assert!(checked >= 13, "the committed table carries its rows");
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn the_determiner_frame_axiom_is_green() {
        use pr4xis::ontology::Axiom as _;
        assert!(DeterminerFrameNeedsNoTypeChanging.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable, Deterministic)]
    #[test]
    fn unary_rules_cost_strictly_more_than_nothing() {
        // Every loaded type-changing row must consume probability mass: this
        // is what terminates the chart's unary fixpoint AND what makes a
        // derivation that skips the rule beat one that uses it, leaf-for-leaf.
        for rule in supertag_cost_table().unary_rules() {
            assert!(rule.cost > DerivationCost::ZERO);
            assert!(rule.from != rule.to);
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn the_shipped_table_carries_no_unary_rows() {
        // The N→NP row was corpus-gate REJECTED (monotonic-or-nothing:
        // define −6 but is-a +12 / non-edge +44, measured 2026-07-10 —
        // .notes/chat-fix-c-build-state.md). This pin makes re-adding a row
        // a CONSCIOUS act that fails a named test until the corpus gate is
        // re-run and this assertion is updated with the new measurement.
        assert!(supertag_cost_table().unary_rules().is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bare_noun_phrase_unary_rule_recomputes_the_published_ratio() {
        // The SAME re-derivation discipline
        // `the_cost_column_is_the_negative_log_of_the_published_ratio` holds
        // the committed TSV rows to, applied to this scoped constructor's
        // cost instead of a TSV cell.
        let rule = bare_noun_phrase_unary_rule();
        assert_eq!(rule.from, LambekType::n());
        assert_eq!(rule.to, LambekType::np());
        let recomputed = -(115_516.0_f64 / 929_552.0_f64).ln();
        assert!((rule.cost.0 - recomputed).abs() < 5e-7);
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn the_reduced_passive_relative_rule_is_the_cited_ccgbank_type_change() {
        // Hockenmaier & Steedman (2005), CCGbank User's Manual MS-CIS-05-09,
        // §3.8 p. 55 (54)a: `S[pss]\NP ⇒ NP\NP` ("workers exposed to it").
        // Repo notation flips the backslash, so the SAME rule reads
        // `NP\S[pss] → NP\NP` here.
        let rule = reduced_passive_relative_unary_rule();
        assert_eq!(rule.from, svo::passive_participle_verb());
        assert_eq!(rule.to, svo::reduced_relative_postmodifier());
        assert_eq!(rule.from.notation(), "NP\\S[pss]");
        assert_eq!(rule.to.notation(), "NP\\NP");
        // HAPAX price: Table 4.3 p. 86 publishes only the top 20 rule
        // instantiations (smallest 8,184) and this rule is not among them,
        // so it is priced at count 1 — the population Table 4.4 records
        // (1,146 rule types occur exactly once) — over the SAME sections
        // 02-21 token total the bare-NP row uses.
        let recomputed = -(1.0_f64 / 929_552.0_f64).ln();
        assert!((rule.cost.0 - recomputed).abs() < 5e-7);
        // Over-priced RELATIVE to every published rule, which is the safety
        // property: a hapax type change can never displace a derivation that
        // parses without it.
        assert!(rule.cost > bare_noun_phrase_unary_rule().cost);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn the_shipped_table_still_carries_no_participial_type_change() {
        // The participial rule is SCOPED (composed on by
        // `statute_structure::grounding::definiens_cost_table`), never
        // loaded — the same discipline `the_shipped_table_carries_no_unary_rows`
        // holds the bare-NP rule to. A global row would change every live
        // chat derivation and is admissible only after a corpus-gate
        // measurement says so.
        assert!(
            supertag_cost_table()
                .unary_rules()
                .iter()
                .all(|r| r.from != svo::passive_participle_verb())
        );
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn with_extra_unary_composes_a_new_table_without_touching_the_shared_one() {
        // The scoped, extended table carries the extra rule...
        let extended =
            supertag_cost_table().with_extra_unary(alloc::vec![bare_noun_phrase_unary_rule()]);
        assert_eq!(extended.unary_rules().len(), 1);
        assert_eq!(extended.unary_rules()[0].from, LambekType::n());
        // ...every lexical row survives the composition...
        assert_eq!(extended.lexical_len(), supertag_cost_table().lexical_len());
        let n_cost = extended.lexical_cost(&svo::noun());
        assert_eq!(n_cost, supertag_cost_table().lexical_cost(&svo::noun()));
        // ...and the SHARED production table is completely untouched — the
        // pinned regression test above still holds after this call.
        assert!(supertag_cost_table().unary_rules().is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    #[should_panic(expected = "duplicate lex row")]
    fn a_duplicate_lex_row_fails_closed() {
        parse_table(
            "lex\tN\tN\t10\t100\t2.302585\nlex\tN\tN\t10\t100\t2.302585\ndefault\t1\t100\t4.605170\n",
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    #[should_panic(expected = "hapax default row")]
    fn a_missing_default_row_fails_closed() {
        parse_table("lex\tN\tN\t10\t100\t2.302585\n");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    #[should_panic(expected = "unrecognized supertag-cost row shape")]
    fn an_unrecognized_row_shape_fails_closed() {
        parse_table("weights\tN\t0.5\n");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    #[should_panic(expected = "strictly positive")]
    fn a_non_positive_cost_fails_closed() {
        parse_table("lex\tN\tN\t100\t100\t0.000000\ndefault\t1\t100\t4.605170\n");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    #[should_panic(expected = "must change the category")]
    fn a_unary_identity_row_fails_closed() {
        parse_table(
            "lex\tN\tN\t10\t100\t2.302585\nunary\tN\tN\tN -> N\t1\t100\t4.605170\ndefault\t1\t100\t4.605170\n",
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    #[should_panic(expected = "does not parse")]
    fn an_unparseable_category_fails_closed() {
        parse_table("lex\tN))\tN\t10\t100\t2.302585\ndefault\t1\t100\t4.605170\n");
    }
}
