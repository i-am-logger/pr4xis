/// Hearing pathology ontology.
///
/// Disorders: conductive, sensorineural, mixed hearing loss, tinnitus,
/// otosclerosis, presbycusis, noise-induced hearing loss, Ménière's disease.
/// Causal graph: damage mechanism → pathological change → perceptual deficit.
///
/// Key references:
/// - Møller 2006: Hearing: Anatomy, Physiology, and Disorders
/// - Gates & Mills 2005: Presbycusis (Lancet)
/// - Henderson et al. 2006: Noise-induced hearing loss
/// - Merchant & Rosowski 2008: Conductive hearing loss
pub mod audiology_functor;
pub mod devices_functor;
pub mod ontology;
