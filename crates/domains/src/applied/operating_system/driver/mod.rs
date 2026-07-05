//! Driver — device drivers and the hardware abstraction layer: the
//! operating-system half of the driver/device seam.
//!
//! Corbet, Rubini & Kroah-Hartman (2005) *Linux Device Drivers* 3e —
//! the driver as the translator of OS requests into device operations,
//! the three device classes, hardware registers, interrupts, and DMA;
//! Liedtke (1995) SOSP — the driver as an isolated user-space server;
//! Swift, Bershad & Levy (2003) SOSP — Nooks fault containment and
//! driver recovery; Ganapathy et al. (2008) ASPLOS — microdrivers;
//! Ryzhyk et al. (2009) SOSP — driver synthesis from a device model.
//!
//! - [`ontology`] — the `Driver` ontology and its four domain axioms;
//!   the fault-containment axiom is discharged against the engine.
//! - [`engine`] — a typed driver/device state machine with fault
//!   injection, after the Nooks containment experiments.
//! - [`dependability_functor`] — the verified `Driver → Dependability`
//!   functor (driver faults land in the Avizienis taxonomy).
//!
//! The other direction of the driver/device seam — `Sensor → Driver`,
//! every sensor is a device a driver drives — lives with its source at
//! [`crate::applied::sensor_fusion::sensor::driver_functor`].

pub mod dependability_functor;
pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
