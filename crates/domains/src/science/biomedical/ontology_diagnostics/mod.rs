/// Meta-ontology: the science of ontology engineering via category theory.
///
/// Formalizes the methodology of detecting missing distinctions in
/// scientific ontologies using adjunctions.
///
/// Novel contribution: adjunctions between domain ontologies automatically
/// reveal where ontological distinctions are missing. Every detected gap
/// corresponds to a published scientific distinction.
///
/// Grounded in:
/// - Spivak & Kent 2012: Ologs (categorical knowledge representation)
/// - Spivak 2014: Category Theory for the Sciences
/// - Mac Lane 1971: Categories for the Working Mathematician
/// - Euzenat & Shvaiko 2013: Ontology Matching
///
/// Novel:
/// - Using adjunction unit/counit for gap detection
/// - ContextDef for gap resolution
/// - Quantifying inter-scale loss via gap ratios
/// - Formalizing the methodology itself as an ontology
pub mod collapse_patterns;
pub mod ontology;
