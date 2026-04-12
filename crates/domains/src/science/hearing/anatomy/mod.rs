/// Auditory anatomy ontology.
///
/// Hierarchy: Ear → Outer/Middle/Inner ear → structures → cells.
/// Mereology: Cochlea has-a BasilarMembrane has-a OrganOfCorti has-a HairCell.
///
/// Key references:
/// - Pickles 2012: An Introduction to the Physiology of Hearing (4th ed.)
/// - Raphael & Altschuler 2003: Structure and innervation of the cochlea
/// - Dallos, Popper & Fay 1996: The Cochlea (Springer)
/// - von Békésy 1960: Experiments in Hearing
pub mod ontology;
pub mod vestibular_functor;
