//! Library of Congress MODS 3.8 — Metadata Object Description Schema.
//!
//! The canonical XML metadata vocabulary GovInfo packages (USREP /
//! SCOTUS-slip / USCOURTS / FR / CFR) use to describe each document at
//! granule level (party names, citation, decision date, justices,
//! syllabus, headnotes, granule structure). Sibling to `lmf/` (Lexical
//! Markup Framework, used by WordNet) and `uslm/` (United States
//! Legislative Markup, used by USC titles) — all three are published
//! XML schemas the praxis runtime loads byte-faithfully from their
//! authoritative sources, with raw-bytes digests pinned in `praxis.lock`.
//!
//! ## Citation
//!
//! - **Library of Congress, Network Development and MARC Standards
//!   Office** (2018) *MODS XML Schema Version 3.8*,
//!   <https://www.loc.gov/standards/mods/v3/mods-3-8.xsd>.
//! - **Library of Congress** (2018) *MODS User Guidelines Version 3*,
//!   <https://www.loc.gov/standards/mods/userguide/>.
//! - **GovInfo / U.S. Government Publishing Office** — MODS as the
//!   metadata format for USREP / SCOTUS-slip / USCOURTS packages,
//!   <https://www.govinfo.gov/help/usreports>.

pub use schema::loaded_mods_3_8;

mod schema;
