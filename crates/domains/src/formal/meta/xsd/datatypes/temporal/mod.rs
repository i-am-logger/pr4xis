//! Lexical / value / canonical mappings for the XSD date/time
//! datatype family (W3C XML Schema 1.1 Part 2 §3.3.6-§3.3.14,
//! §3.4.26-§3.4.28).
//!
//! All members share the seven-property model (§D.2.1) and the
//! fragment maps in [`common`]. [`dates`] covers the date and
//! Gregorian projections (`date`, `gYearMonth`, `gYear`, `gMonthDay`,
//! `gDay`, `gMonth`).
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05.

pub mod common;
pub mod dates;
pub mod duration;
pub mod times;
