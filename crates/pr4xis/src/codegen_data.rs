/// Language-agnostic data produced by codegen.
///
/// This is the standard interface between build-time code generation
/// and runtime Language construction. Codegen does not know English
/// or any specific language — it produces this generic structure.
/// The Language functor (English, Hebrew, etc.) consumes it.
///
/// Always available (no feature flag). The codegen module that
/// generates this data requires the `codegen` feature, but consuming
/// the output does not.
pub struct CodegenData {
    pub entity_count: usize,
    pub entity_ids: &'static [&'static str],
    pub entity_pos: &'static [&'static str],
    pub entity_labels: &'static [&'static str],
    pub entity_defs: &'static [&'static str],
    pub word_index: &'static [(&'static str, &'static [u32])],
    pub taxonomy: &'static [(u32, u32)],
    pub mereology: &'static [(u32, u32)],
    pub opposition: &'static [(u32, u32)],
    pub equivalence: &'static [(u32, u32)],
    pub causation: &'static [(u32, u32)],
}

impl CodegenData {
    /// Look up concept indices by word text (binary search on sorted word_index).
    pub fn lookup(&self, word: &str) -> &'static [u32] {
        match self.word_index.binary_search_by_key(&word, |(w, _)| w) {
            Ok(idx) => self.word_index[idx].1,
            Err(_) => &[],
        }
    }
}
