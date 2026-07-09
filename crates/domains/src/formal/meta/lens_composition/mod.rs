//! Asymmetric well-behaved lenses and their sequential composition
//! (Foster, Greenwald, Moore, Pierce & Schmitt 2007).
//!
//! [`Lens`] is the `Source ⇆ View` pair `(get, put)` satisfying the
//! GetPut / PutGet / PutPut laws (§3); [`Compose`] is the sequential
//! composite `l ; k` (§3); [`IdentityLens`] is its unit. Together the
//! lenses form a category, the substrate for the
//! `file ⇄ xml ⇄ xsd ⇄ statute` pipeline — each hop a lens, the whole
//! its composite.
//!
//! Distinct from [`crate::formal::meta::well_behaved_lens`], which is
//! the *byte-anchored* file-reading witness (`bytes ⇄ ontology`,
//! PutGet up to canonical form). This module is the *general*
//! source-to-view lens that composes across the typed layers above the
//! byte boundary.
//!
//! ## Citation
//!
//! - **Foster, J. N., Greenwald, M. B., Moore, J. T., Pierce, B. C. &
//!   Schmitt, A.** "Combinators for Bidirectional Tree
//!   Transformations", *ACM TOPLAS* 29(3), 2007. §3 (Definition 3.2).

pub mod lens;

#[doc(inline)]
pub use lens::{
    Compose, ComposeError, IdentityLens, Lens, WellBehavedLensAdapter, get_put_holds,
    put_get_holds, put_put_holds,
};
