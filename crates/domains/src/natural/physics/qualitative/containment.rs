//! Realized naive-physics containment/support mechanics (Hayes 1979 *The
//! Naive Physics Manifesto*; Hayes 1985 *Naive Physics I: Ontology for
//! Liquids* §3): a qualitative SIZE ordering over individuals, the
//! container-fits-content constraint, and the support-or-falls principle.

use alloc::string::String;

/// A qualitative, ordered size magnitude — Hayes (1985) §3 treats container
/// capacity qualitatively, not as an exact volume. `Ord` gives the total
/// order a QuantitySpace-style comparison needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    Small,
    Medium,
    Large,
}

/// A physical object or substance — Hayes (1985) §2's "piece of stuff" /
/// Forbus (1984)'s Individual, specialized here with the Size a
/// containment judgment needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Individual {
    pub name: String,
    pub size: Size,
}

/// Does `container` have room for `content`? Hayes (1985) §3: a container's
/// capacity must be at least as large as what it holds — the physical
/// constraint licensing "won't fit" reasoning (the classic commonsense
/// containment inference).
pub fn fits(container: &Individual, content: &Individual) -> bool {
    container.size >= content.size
}

/// Which individual is "too big", given a FAILED containment attempt
/// (`container` cannot hold `content`)? The Winograd-schema antecedent
/// (Levesque, Davis & Morgenstern 2012) derived DIRECTLY from the size
/// ordering [`fits`] states — the one whose size EXCEEDS the other's.
/// `None` when the attempt does not fail (nothing to explain).
pub fn too_big<'a>(container: &'a Individual, content: &'a Individual) -> Option<&'a Individual> {
    (!fits(container, content)).then_some(content)
}

/// Which individual is "too small", given the SAME failed containment
/// attempt — the complementary Winograd-schema twin (Sakaguchi et al. 2020
/// WinoGrande's "twin sentences" swap the predicated adjective, not the
/// underlying situation): the one whose size falls SHORT.
pub fn too_small<'a>(container: &'a Individual, content: &'a Individual) -> Option<&'a Individual> {
    (!fits(container, content)).then_some(container)
}

/// Does `supported` stay up, given whether it currently has support? Hayes
/// (1979) *The Naive Physics Manifesto* — the foundational naive-physics
/// claim that a resting object needs support; without one, it falls
/// (formalized by Davis 1990 *Representations of Commonsense Knowledge*
/// Ch. 7's support axioms).
pub fn falls_without_support(has_support: bool) -> bool {
    !has_support
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small(name: &str) -> Individual {
        Individual {
            name: String::from(name),
            size: Size::Small,
        }
    }
    fn large(name: &str) -> Individual {
        Individual {
            name: String::from(name),
            size: Size::Large,
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_container_at_least_as_large_fits_its_content() {
        let suitcase = large("suitcase");
        let trophy = small("trophy");
        assert!(fits(&suitcase, &trophy));
        assert_eq!(too_big(&suitcase, &trophy), None);
        assert_eq!(too_small(&suitcase, &trophy), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_winograd_pair_swaps_antecedent_with_the_predicated_adjective() {
        // "The trophy doesn't fit in the suitcase because it is too big."
        //   -> "it" = trophy (the content whose size exceeds the container).
        // "The trophy doesn't fit in the suitcase because it is too small."
        //   -> "it" = suitcase (the container whose size falls short).
        // BOTH antecedents fall out of the SAME failed-fits judgment; only
        // which of `too_big`/`too_small` is asked determines which entity
        // answers — no discourse/centering computation, per the axiom.
        let suitcase = small("suitcase");
        let trophy = large("trophy");
        assert!(!fits(&suitcase, &trophy));
        assert_eq!(
            too_big(&suitcase, &trophy).map(|i| i.name.as_str()),
            Some("trophy")
        );
        assert_eq!(
            too_small(&suitcase, &trophy).map(|i| i.name.as_str()),
            Some("suitcase")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn equal_sizes_fit() {
        let a = small("box a");
        let b = small("box b");
        assert!(fits(&a, &b));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn an_unsupported_object_falls() {
        assert!(falls_without_support(false));
        assert!(!falls_without_support(true));
    }
}
