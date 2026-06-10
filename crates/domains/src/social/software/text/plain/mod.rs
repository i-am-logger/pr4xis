//! Plain UTF-8 text as a byte-exact, graph-faithful well-behaved lens
//! (M4.ι / #186, Phase 1).
//!
//! - [`ontology`] — the concrete-syntax decisions that fix a plain-text
//!   document's bytes (BOM, per-line terminator, final incomplete line),
//!   each citing its authority (Unicode §5.8, BOM FAQ, POSIX §3.195/§3.206).
//! - [`lens`] — [`lens::PlainTextLens`], which reconstructs the exact
//!   input bytes from that ontology graph (`put(get(b)) == b`) with no
//!   constant-complement.

pub mod lens;
pub mod ontology;

#[cfg(test)]
mod tests;
