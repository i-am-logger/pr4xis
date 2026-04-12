/// Disease pathology ontology — pure science of disease classification and progression.
///
/// Models disease states, staging, classification, and pathological processes
/// as formal ontology:
/// - Disease states: normal tissue through neoplasia
/// - Staging: low-grade and high-grade dysplasia
/// - Classification: benign, premalignant, malignant
/// - Processes: inflammation, cellular adaptation, atypical growth, invasion
/// - Causal chains: tissue insult -> acute response -> chronic adaptation -> metaplasia -> dysplasia -> neoplasia
/// - Fibrotic pathway: chronic adaptation -> fibrotic remodeling -> stricture formation
///
/// Key references:
/// - Levin 2014: depolarized Vmem correlates with neoplastic transformation
/// - Chernet & Levin 2013: repolarization suppresses tumor formation
/// - Binns et al. 2019: bioelectric control of metaplasia reversal
pub mod bioelectricity_functor;
pub mod ontology;
