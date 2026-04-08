/// Prime numbers and the Sieve of Eratosthenes.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n.is_multiple_of(i) || n.is_multiple_of(i + 2) {
            return false;
        }
        i += 6;
    }
    true
}

/// Sieve of Eratosthenes: all primes up to n.
pub fn sieve(n: usize) -> Vec<u64> {
    let mut is_p = vec![true; n + 1];
    is_p[0] = false;
    if n > 0 {
        is_p[1] = false;
    }
    for i in 2..=((n as f64).sqrt() as usize) {
        if is_p[i] {
            for j in (i * i..=n).step_by(i) {
                is_p[j] = false;
            }
        }
    }
    is_p.iter()
        .enumerate()
        .filter(|&(_, &p)| p)
        .map(|(i, _)| i as u64)
        .collect()
}

/// Goldbach's conjecture (unproven but verified): every even n > 2 is sum of two primes.
pub fn goldbach(n: u64) -> Option<(u64, u64)> {
    if n <= 2 || !n.is_multiple_of(2) {
        return None;
    }
    for p in sieve(n as usize) {
        if p > n / 2 {
            break;
        }
        if is_prime(n - p) {
            return Some((p, n - p));
        }
    }
    None
}

/// Fundamental theorem of arithmetic: unique prime factorization.
pub fn factorize(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();
    let mut d = 2;
    while d * d <= n {
        while n.is_multiple_of(d) {
            factors.push(d);
            n /= d;
        }
        d += 1;
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_small_primes() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(97));
    }

    #[test]
    fn test_sieve() {
        let primes = sieve(30);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn test_goldbach() {
        for n in (4..100).step_by(2) {
            let (a, b) = goldbach(n).unwrap_or_else(|| panic!("Goldbach failed for {}", n));
            assert_eq!(a + b, n);
            assert!(is_prime(a));
            assert!(is_prime(b));
        }
    }

    #[test]
    fn test_factorize() {
        assert_eq!(factorize(12), vec![2, 2, 3]);
        assert_eq!(factorize(97), vec![97]);
        assert_eq!(factorize(100), vec![2, 2, 5, 5]);
    }

    proptest! {
        /// Product of factors equals original
        #[test]
        fn prop_factorization_product(n in 2..10000u64) {
            let factors = factorize(n);
            let product: u64 = factors.iter().product();
            prop_assert_eq!(product, n);
        }

        /// All factors are prime
        #[test]
        fn prop_factors_are_prime(n in 2..10000u64) {
            for f in factorize(n) {
                prop_assert!(is_prime(f), "{} is not prime (factor of {})", f, n);
            }
        }

        /// Sieve agrees with is_prime
        #[test]
        fn prop_sieve_consistent(n in 2..1000usize) {
            let primes = sieve(n);
            for i in 2..=n {
                prop_assert_eq!(primes.contains(&(i as u64)), is_prime(i as u64), "mismatch for {}", i);
            }
        }

        /// Goldbach: every even n > 2 has a decomposition (verified up to input)
        #[test]
        fn prop_goldbach(n in (2..500u64).prop_map(|n| n * 2)) {
            prop_assume!(n > 2);
            let result = goldbach(n);
            prop_assert!(result.is_some(), "Goldbach failed for {}", n);
            let (a, b) = result.unwrap();
            prop_assert_eq!(a + b, n);
            prop_assert!(is_prime(a));
            prop_assert!(is_prime(b));
        }

        /// Primes > 2 are odd
        #[test]
        fn prop_primes_odd(n in 3..10000u64) {
            if is_prime(n) {
                prop_assert!(n % 2 != 0, "prime {} is even", n);
            }
        }
    }
}
