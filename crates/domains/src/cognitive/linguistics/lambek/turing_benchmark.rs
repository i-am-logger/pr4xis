// Turing-test taxonomy questions answered through the WordNet
// is-a backbone. Questions drawn from the Loebner Prize transcripts
// (Shieber 1994 *Communications of the ACM* 37.6) — these are the
// L1 baseline a praxis run can answer today via the existing
// English/WordNet ontology, with no additional substrate.
//
// Larger research areas the Turing-test corpus exercises — pregroup
// grammar pipeline, geography, literature, mereology + counting,
// arithmetic, Winograd schema, material physics, phatic dialogue,
// self-model — are tracked in the project memory entry
// `project-turing-benchmark-research-areas`, not as ignored tests
// here.

#[cfg(test)]
mod tests {
    use crate::cognitive::linguistics::english::{English, english_loaded};

    /// Full English via the shared `english_loaded()` fast path (the compact
    /// `.prx` archive when present, else the WN-LMF parse) — parsed once per
    /// process rather than re-read fresh on every call.
    fn english() -> &'static English {
        english_loaded()
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn taxonomy_is_a_dog_a_mammal() {
        let en = english();
        let dog = en.lookup("dog");
        let mammal = en.lookup("mammal");
        assert!(!dog.is_empty() && !mammal.is_empty());
        let mut found = false;
        for &d in dog {
            for &m in mammal {
                if en.is_a(d, m) {
                    found = true;
                }
            }
        }
        assert!(found, "dog should be a mammal");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn taxonomy_is_a_dog_an_animal() {
        let en = english();
        let dog = en.lookup("dog");
        let animal = en.lookup("animal");
        assert!(!dog.is_empty() && !animal.is_empty());
        let mut found = false;
        for &d in dog {
            for &a in animal {
                if en.is_a(d, a) {
                    found = true;
                }
            }
        }
        assert!(found, "dog should be an animal");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn taxonomy_what_is_a_dog() {
        let en = english();
        let ids = en.lookup("dog");
        assert!(!ids.is_empty());
        let concept = en.concept(ids[0]).unwrap();
        assert!(
            concept.definitions().next().is_some(),
            "dog should have a definition"
        );
    }
}
