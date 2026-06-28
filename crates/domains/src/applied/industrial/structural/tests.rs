use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::industrial::structural::engine::*;
use crate::applied::industrial::structural::ontology::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn structural_category_laws() {
    assert_category_laws::<StructuralCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn structural_ontology_validates() {
    StructuralOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn strain_bounded_elastic_holds() {
    assert!(StrainBoundedElastic.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn crack_monotonicity_holds() {
    assert!(CrackMonotonicity.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn elastic_within_yield() {
    assert!(is_elastic(1000.0, 2000.0));
    assert!(is_elastic(-1000.0, 2000.0));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn plastic_beyond_yield() {
    assert!(!is_elastic(2500.0, 2000.0));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn damage_index_none_severity() {
    let readings = vec![StrainReading {
        microstrain: 100.0,
        sensor_id: 0,
        timestamp: 0.0,
    }];
    let di = compute_damage_index(&readings, 2000.0).unwrap();
    assert_eq!(di.severity, DamageSeverity::None);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn damage_index_critical_severity() {
    let readings = vec![StrainReading {
        microstrain: 2500.0,
        sensor_id: 0,
        timestamp: 0.0,
    }];
    let di = compute_damage_index(&readings, 2000.0).unwrap();
    assert_eq!(di.severity, DamageSeverity::Critical);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn damage_index_empty_returns_none() {
    let di = compute_damage_index(&[], 2000.0);
    assert!(di.is_none());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rms_strain_constant() {
    let readings: Vec<StrainReading> = (0..10)
        .map(|i| StrainReading {
            microstrain: 500.0,
            sensor_id: 0,
            timestamp: i as f64,
        })
        .collect();
    let rms = rms_strain(&readings);
    assert!((rms - 500.0).abs() < 1e-10);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn rms_strain_empty() {
    assert!((rms_strain(&[]) - 0.0).abs() < 1e-12);
}

// ---------------------------------------------------------------------------
// H8: NaN microstrain does not panic in compute_damage_index
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Honest)]
#[test]
fn compute_damage_index_nan_strain_no_panic() {
    let readings = vec![
        StrainReading {
            microstrain: f64::NAN,
            sensor_id: 0,
            timestamp: 0.0,
        },
        StrainReading {
            microstrain: 500.0,
            sensor_id: 1,
            timestamp: 1.0,
        },
    ];
    // Should not panic
    let di = compute_damage_index(&readings, 2000.0);
    assert!(di.is_some());
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn severity_monotonic_with_ratio(ratio in 0.0..2.0_f64) {
            let severity = classify_severity(ratio);
            // Higher ratio should give at least as severe classification
            let severity_higher = classify_severity(ratio + 0.1);
            let to_num = |s: DamageSeverity| match s {
                DamageSeverity::None => 0,
                DamageSeverity::Minor => 1,
                DamageSeverity::Moderate => 2,
                DamageSeverity::Severe => 3,
                DamageSeverity::Critical => 4,
            };
            prop_assert!(to_num(severity_higher) >= to_num(severity),
                "severity should be monotonic: ratio={}, s={:?}; ratio+0.1 s={:?}",
                ratio, severity, severity_higher);
        }

        #[test]
        fn rms_strain_non_negative(
            strains in proptest::collection::vec(-5000.0..5000.0_f64, 1..20)
        ) {
            let readings: Vec<StrainReading> = strains.iter().enumerate()
                .map(|(i, &s)| StrainReading { microstrain: s, sensor_id: 0, timestamp: i as f64 })
                .collect();
            prop_assert!(rms_strain(&readings) >= 0.0);
        }
    }

    pr4xis::register_praxis_value!(severity_monotonic_with_ratio, Verifiable);
    pr4xis::register_praxis_value!(rms_strain_non_negative, Verifiable);
}
