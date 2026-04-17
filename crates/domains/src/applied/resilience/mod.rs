//! Resilience ontology — recovery patterns, backoff, supervision.
//!
//! Companion to `applied::dependability` (#122). Where Dependability defines
//! what errors ARE, Resilience defines how systems RECOVER: stability patterns
//! (Nygard 2007), backoff strategies (Brooker 2015; Metcalfe & Boggs 1976),
//! supervision trees (Armstrong 2003 Erlang/OTP), and recovery-oriented
//! computing (Patterson et al. 2002).

pub mod ontology;
