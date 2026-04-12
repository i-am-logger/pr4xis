/// Bone conduction ontology.
///
/// How vibration reaches the cochlea via skull bone rather than air.
/// Three primary mechanisms: osseotympanic, inertial, compressional.
/// Causal graph: transducer → skull vibration → cochlear stimulation.
///
/// Key references:
/// - Stenfelt & Goode 2005: Bone-conducted sound: physiological and clinical aspects
/// - Stenfelt 2011: Acoustic and physiologic aspects of bone conduction hearing
/// - Tonndorf 1966: Bone conduction. Studies in experimental animals
/// - von Békésy 1960: Experiments in Hearing
/// - Stenfelt 2015: Inner ear contribution to bone conduction hearing
pub mod acoustics_functor;
pub mod ontology;
