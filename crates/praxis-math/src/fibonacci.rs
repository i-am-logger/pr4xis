/// Fibonacci sequence: F(0)=0, F(1)=1, F(n)=F(n-1)+F(n-2).
pub fn fib(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 2..=n {
        let next = a.saturating_add(b);
        a = b;
        b = next;
    }
    b
}

/// Golden ratio: F(n+1)/F(n) → φ = (1+√5)/2
pub fn golden_ratio() -> f64 {
    (1.0 + 5.0_f64.sqrt()) / 2.0
}

/// Binet's formula: F(n) = (φⁿ - ψⁿ)/√5 where ψ = (1-√5)/2.
pub fn binet(n: u64) -> f64 {
    let phi = golden_ratio();
    let psi = (1.0 - 5.0_f64.sqrt()) / 2.0;
    (phi.powi(n as i32) - psi.powi(n as i32)) / 5.0_f64.sqrt()
}

/// Cassini's identity: F(n-1)*F(n+1) - F(n)² = (-1)^n
pub fn cassini(n: u64) -> i64 {
    if n == 0 {
        return 0;
    }
    let fn_1 = fib(n - 1) as i64;
    let fn_ = fib(n) as i64;
    let fn1 = fib(n + 1) as i64;
    fn_1 * fn1 - fn_ * fn_
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_first_10() {
        let expected = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
        for (i, &e) in expected.iter().enumerate() {
            assert_eq!(fib(i as u64), e);
        }
    }

    #[test]
    fn test_golden_ratio() {
        let phi = golden_ratio();
        assert!((phi - 1.618033988749895).abs() < 1e-10);
    }

    #[test]
    fn test_cassini_identity() {
        for n in 1..20 {
            let result = cassini(n);
            let expected = if n % 2 == 1 { -1 } else { 1 }; // (-1)^n
            assert_eq!(result, expected, "Cassini failed for n={}", n);
        }
    }

    proptest! {
        /// F(n) = F(n-1) + F(n-2) for n >= 2
        #[test]
        fn prop_recurrence(n in 2..30u64) {
            prop_assert_eq!(fib(n), fib(n - 1) + fib(n - 2));
        }

        /// Cassini's identity: F(n-1)*F(n+1) - F(n)² = (-1)^n
        #[test]
        fn prop_cassini(n in 1..25u64) {
            let expected = if n % 2 == 1 { -1i64 } else { 1 }; // (-1)^n
            prop_assert_eq!(cassini(n), expected);
        }

        /// F(n+1)/F(n) approaches golden ratio
        #[test]
        fn prop_golden_ratio_convergence(n in 10..30u64) {
            let ratio = fib(n + 1) as f64 / fib(n) as f64;
            prop_assert!((ratio - golden_ratio()).abs() < 0.001);
        }

        /// Binet's formula: F(n) = (φⁿ - ψⁿ)/√5 exactly matches fib(n)
        #[test]
        fn prop_binet_matches_fib(n in 0..25u64) {
            let exact = fib(n) as f64;
            let binet_val = binet(n);
            prop_assert!((binet_val - exact).abs() < 0.5,
                "binet({})={} ≠ fib({})={}", n, binet_val, n, exact);
        }

        /// F(n) is monotonically non-decreasing
        #[test]
        fn prop_monotonic(n in 1..50u64) {
            prop_assert!(fib(n) >= fib(n - 1));
        }
    }
}
