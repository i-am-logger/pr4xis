/// Bayes' theorem: P(A|B) = P(B|A) * P(A) / P(B).
///
/// The foundation of all estimation theory.
/// In sensor fusion: posterior = likelihood * prior / evidence.
///
/// Source: Bayes, T. (1763). "An Essay towards solving a Problem
///         in the Doctrine of Chances." Philosophical Transactions.
///         Kolmogorov (1933). Axiom 3 + conditional probability definition.
pub fn bayes(prior: f64, likelihood: f64, evidence: f64) -> f64 {
    assert!(evidence > 0.0, "evidence P(B) must be positive");
    likelihood * prior / evidence
}

/// Compute evidence (marginal likelihood) from prior and likelihood
/// over a discrete hypothesis space.
///
/// P(B) = Σ_i P(B|H_i) * P(H_i)
pub fn evidence(priors: &[f64], likelihoods: &[f64]) -> f64 {
    assert_eq!(priors.len(), likelihoods.len());
    priors.iter().zip(likelihoods).map(|(p, l)| p * l).sum()
}

/// Full Bayesian update over a discrete hypothesis space.
///
/// Given priors P(H_i) and likelihoods P(D|H_i), compute posteriors P(H_i|D).
/// Returns None if priors don't sum to 1 or any are negative.
pub fn bayesian_update(priors: &[f64], likelihoods: &[f64]) -> Option<Vec<f64>> {
    assert_eq!(priors.len(), likelihoods.len());

    // Validate priors form a valid distribution
    if priors.iter().any(|&p| p < 0.0) {
        return None;
    }
    let prior_sum: f64 = priors.iter().sum();
    if (prior_sum - 1.0).abs() > 1e-10 {
        return None;
    }

    let ev = evidence(priors, likelihoods);
    if ev < 1e-15 {
        return None; // zero evidence
    }

    let posteriors: Vec<f64> = priors
        .iter()
        .zip(likelihoods)
        .map(|(p, l)| p * l / ev)
        .collect();

    Some(posteriors)
}

/// Log-odds form of Bayes' theorem (numerically stable for extreme probabilities).
///
/// log_odds(posterior) = log_odds(prior) + log_likelihood_ratio
///
/// where log_odds(p) = ln(p / (1-p))
pub fn log_odds(p: f64) -> f64 {
    (p / (1.0 - p)).ln()
}

/// Convert log-odds back to probability.
pub fn from_log_odds(lo: f64) -> f64 {
    1.0 / (1.0 + (-lo).exp())
}

/// Bayesian update in log-odds form (binary hypothesis).
pub fn log_odds_update(prior_log_odds: f64, log_likelihood_ratio: f64) -> f64 {
    prior_log_odds + log_likelihood_ratio
}
