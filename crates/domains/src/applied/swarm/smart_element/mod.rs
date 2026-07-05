//! SmartElement — the autonomic edge element: a MAPE-K manager that is
//! also a signed-estimate fusion peer, carrying a queryable local ontology.
//!
//! The autonomic loop (Kephart & Chess 2003) and the information-form
//! estimation math (Maybeck 1979; Mutambara 1998; Olfati-Saber, Fax &
//! Murray 2007) are established literature — this module encodes them and
//! claims no novelty for them. The **only** novelty claimed is the
//! ontological synthesis: an autonomic driver that is *simultaneously* a
//! MAPE-K element and a signed-estimate fusion peer, with fusion
//! consistency as cited axioms and the whole thing `no_std` at the edge.
//!
//! - [`ontology`] — the `SmartElement` ontology, three smartness
//!   predicates, and five domain axioms (each discharged against the
//!   engine or a functor image).
//! - [`engine`] — the autonomic loop, pure `no_std`: the MAPE cycle over a
//!   local [`InformationEstimate`](crate::applied::sensor_fusion::state::information::InformationEstimate)
//!   with exclusion-before-aggregation of equivocators.
//! - [`mape_k_functor`] — `SmartElement → MapeK`: the element IS an
//!   autonomic loop (the image covers every phase).
//! - [`sensor_functor`] — `SmartElement → Sensor`: the forgetful
//!   smart-transducer reading.
//! - [`consensus_functor`] — `SmartElement → Consensus`: the element is a
//!   fusion `Peer` (the faithful anchor of `SmartIsFusionPeer`).
//! - [`driver_functor`] — `SmartElement → Driver`: the synthesis anchor —
//!   `SmartDriver → Driver`, `Teds → DeviceModel`, `SelfHealing → Recovery`.
//! - [`dependability_functor`] — `SmartElement → Dependability`:
//!   `SelfHealing → ErrorRecovery`, `SelfProtection → FaultHandling`.
//! - [`constitutive_functor`] — `SmartElement → ConstitutiveProtocol`: the
//!   element signs under an `Identity`; `SelfProtection → Slashing`.

pub mod consensus_functor;
pub mod constitutive_functor;
pub mod dependability_functor;
pub mod driver_functor;
pub mod engine;
pub mod mape_k_functor;
pub mod ontology;
pub mod sensor_functor;

#[cfg(test)]
mod tests;
