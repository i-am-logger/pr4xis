//! W3C XML 1.0 EBNF grammar — runtime types + parser for the
//! Notation defined in Appendix B of the spec.
//!
//! This module is the **substrate** for an ontologically-grounded XML
//! 1.0 parser. The grammar itself is loaded from the published spec
//! at runtime via [`load_grammar`]; the per-production parsing logic
//! is a single literature-grounded interpreter (the M5.ζ.3 follow-up,
//! not yet present in this module).
//!
//! Per `feedback_bottom_up_loaded_not_encoded`, the grammar shape is
//! **never** hand-coded in Rust. The only Rust grammar code in the
//! world we're building toward is the generic PEG interpreter — it
//! consumes the loaded productions.
//!
//! ## Literature stack
//!
//! Theoretical foundations for the runtime semantics (Ford's PEG
//! formalism + Packrat memoisation):
//!
//! - **Ford, B.** (2002) *Packrat Parsing: a Practical Linear-Time
//!   Algorithm with Backtracking*, MIT Master's thesis (Sept 2002).
//!   The linear-time-guarantee proof. <https://pdos.csail.mit.edu/~baford/packrat/thesis/>.
//! - **Ford, B.** (2002) "Packrat Parsing: Simple, Powerful, Lazy,
//!   Linear Time", *ICFP '02* — the functional pearl.
//! - **Ford, B.** (2004) "Parsing Expression Grammars: A
//!   Recognition-Based Syntactic Foundation", *POPL '04* §2.
//!
//! W3C tradition for parsers driven by spec EBNF:
//!
//! - **Berners-Lee, T.** Python predictive parser at
//!   `www.w3.org/2000/10/swap/grammar/bnf` — the original W3C EBNF
//!   interpreter.
//! - **Connolly, D.** *bnf2turtle* (MadMode, 2006) — EBNF → RDF
//!   Turtle processor.
//! - **Prud'hommeaux, E.** *Yacker* (developed during SPARQL
//!   standardization) — EBNF → yacc converter with interactive
//!   string-against-grammar checking.
//! - **dryruby/ebnf** (Ruby gem) — the closest open-source precedent
//!   for what we're building. Handles the W3C XML 1.0 EBNF dialect
//!   including set subtraction (`A - B`), runs both PEG+Packrat and
//!   LL(1) modes, both interprets at runtime and emits code.
//!
//! The EBNF notation itself:
//!
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (eds.) (2008) *Extensible Markup Language (XML)
//!   1.0 (Fifth Edition)*, W3C Recommendation 26 November 2008,
//!   **Appendix B** *Notation*.
//! - **Wirth, N.** (1977) "What can we do about the unnecessary
//!   diversity of notation for syntactic definitions?" *Comm. ACM*
//!   20(11):822-823 — historical foundation.
//! - **ISO/IEC 14977:1996** *Extended BNF*.

pub use ast::{CodePointRange, Grammar, Production, Term};
pub use interpreter::{Interpreter, MatchResult};
pub use loader::{LoadGrammarError, load_grammar};
pub use rhs_parser::{ParseRhsError, parse_rhs};

mod ast;
mod interpreter;
mod loader;
mod rhs_parser;
